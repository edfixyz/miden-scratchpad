use miden_lib::account::auth::{AuthRpoFalcon512, NoAuth};
use rand::{prelude::StdRng, Rng, RngCore, SeedableRng};
use std::{fs, path::Path, sync::Arc};

mod helper;
use helper::EDFI_BANNER;

use miden_assembly::{
    LibraryPath,
    ast::{Module, ModuleKind},
};
use miden_client::{
    Client, ClientError, Felt, ScriptBuilder,
    account::{
        Account, AccountBuilder, AccountIdAddress, AccountStorageMode, AccountType,
        Address, AddressInterface, StorageSlot, component::BasicWallet, StorageMap,
    },
    auth::AuthSecretKey,
    auth::TransactionAuthenticator,
    builder::ClientBuilder,
    crypto::SecretKey,
    keystore::FilesystemKeyStore,
    note::{
        Note, NoteAssets, NoteExecutionHint, NoteExecutionMode, NoteInputs, NoteMetadata,
        NoteRecipient, NoteTag, NoteType,
    },
    rpc::{Endpoint, TonicRpcClient},
    transaction::{OutputNote, TransactionKernel, TransactionRequestBuilder},
};
use miden_objects::{
    Word,
    account::{AccountComponent, NetworkId},
    assembly::Assembler,
    assembly::DefaultSourceManager,
};

fn create_library(
    assembler: Assembler,
    library_path: &str,
    source_code: &str,
) -> Result<miden_assembly::Library, Box<dyn std::error::Error>> {
    let source_manager = Arc::new(DefaultSourceManager::default());
    let module = Module::parser(ModuleKind::Library).parse_str(
        LibraryPath::new(library_path)?,
        source_code,
        &source_manager,
    )?;
    let library = assembler.clone().assemble_library([module])?;
    Ok(library)
}

async fn consume_note<AUTH: TransactionAuthenticator + Sync + 'static>(
    client: &mut Client<AUTH>,
    note: &Note,
    desk_account: &Account,
) -> Result<(), ClientError> {
    // let secret = Word::from([3u8, 3, 3, 3]);
    let consume_custom_request = TransactionRequestBuilder::new()
        .unauthenticated_input_notes([(note.clone(), None)])
        .build()
        .unwrap();
    let tx_result = client
        .new_transaction(desk_account.id(), consume_custom_request)
        .await
        .unwrap();
    let tx_id = tx_result.executed_transaction().id();
    println!(
        "Consuming note tx on MidenScan: https://testnet.midenscan.com/tx/{:?} \n",
        tx_id
    );
    client.submit_transaction(tx_result).await?;
    Ok(())
}

async fn create_note<AUTH: TransactionAuthenticator + Sync + 'static>(
    client: &mut Client<AUTH>,
    offer_rng: &mut StdRng,
    uuid_rng: &mut StdRng,
    account: &Account,
    _desk_account: &Account,
) -> Result<Note, ClientError> {
    let code = fs::read_to_string(Path::new("./masm/notes/limit_buy_request.masm")).unwrap();
    let book_code = fs::read_to_string(Path::new("./masm/accounts/book.masm")).unwrap();
    let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);
    let book_component_lib = create_library(assembler, "external_contract::book", &book_code).unwrap();
    let note_script = ScriptBuilder::new(true).with_dynamically_linked_library(&book_component_lib).unwrap().compile_note_script(code).unwrap();

    let secret = Word::from([3u8, 3, 3, 3]);
    
    // Generate random amount between 10 and 100
    let amount: u64 = offer_rng.random_range(10..=100);
    
    // Generate random price between 99000 and 150000
    let price: u64 = offer_rng.random_range(99000..=150000);

    println!("\nBTCUSD Market");
    println!("Offer amount: {} price: {}", amount, price);
    let uuid: u128 = uuid_rng.random();
    let uuid_high = (uuid >> 64) as u64;
    let uuid_low = uuid as u64;
    let inputs = vec![Felt::new(uuid_high), Felt::new(uuid_low), Felt::new(0), Felt::new(0), Felt::new(amount), Felt::new(price)];
    let note_inputs = NoteInputs::new(inputs)?;
    let recipient = NoteRecipient::new(secret, note_script, note_inputs);
    let tag = NoteTag::for_public_use_case(0, 0, NoteExecutionMode::Local).unwrap();
    let metadata = NoteMetadata::new(
        account.id(),
        NoteType::Public,
        tag,
        NoteExecutionHint::always(),
        Felt::new(0),
    )?;
    let assets = NoteAssets::new(vec![])?;
    let custom_note = Note::new(assets, metadata, recipient);
    let note_req = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(custom_note.clone())])
        .build()
        .unwrap();
    let tx_result = client
        .new_transaction(account.id(), note_req)
        .await
        .unwrap();
    let tx_id = tx_result.executed_transaction().id();
    client.submit_transaction(tx_result).await?;
    println!(
        "View transaction on MidenScan: https://testnet.midenscan.com/tx/{:?}",
        tx_id
    );
    Ok(custom_note)
}

async fn create_basic_account(
    client: &mut Client<FilesystemKeyStore<rand::prelude::StdRng>>,
) -> Result<Account, ClientError> {
    let keystore: FilesystemKeyStore<StdRng> =
        FilesystemKeyStore::new("./keystore".into()).unwrap().into();
    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);
    let key_pair = SecretKey::with_rng(client.rng());
    let builder = AccountBuilder::new(init_seed)
        .account_type(AccountType::RegularAccountUpdatableCode)
        .storage_mode(AccountStorageMode::Public)
        .with_auth_component(AuthRpoFalcon512::new(key_pair.public_key()))
        .with_component(BasicWallet);
    let (account, seed) = builder.build().unwrap();
    client.add_account(&account, Some(seed), false).await?;
    keystore
        .add_key(&AuthSecretKey::RpoFalcon512(key_pair))
        .unwrap();
    Ok(account)
}

async fn deploy_book_account<AUTH: TransactionAuthenticator>(
    client: &mut Client<AUTH>,
) -> Result<Account, ClientError> {

    // Prepare assembler (debug mode = true)
    let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);

    // Load the MASM file for the counter contract
    let book_path = Path::new("./masm/accounts/book.masm");
    let book_code = fs::read_to_string(book_path).unwrap();

    // Compile the account code into `AccountComponent` with one storage slot
    let counter_component = AccountComponent::compile(
        book_code.clone(),
        assembler,
        vec![StorageSlot::Value(
            [Felt::new(0), Felt::new(0), Felt::new(0), Felt::new(0)].into(),
        ), 
        StorageSlot::Map(StorageMap::new()),
        StorageSlot::Value(
            [Felt::new(0), Felt::new(0), Felt::new(0), Felt::new(0)].into(),
        ),
        ],
    )
    .unwrap()
    .with_supports_all_types();

    let mut seed = [0u8; 32];
    client.rng().fill_bytes(&mut seed);

    // Build the new `Account` with the component
    let (book_contract, counter_seed) = AccountBuilder::new(seed)
        .account_type(AccountType::RegularAccountImmutableCode)
        .storage_mode(AccountStorageMode::Public)
        .with_component(counter_component.clone())
        .with_auth_component(NoAuth)
        .build()
        .unwrap();

    client
        .add_account(&book_contract.clone(), Some(counter_seed), false)
        .await?;

    Ok(book_contract)
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    println!("{}", EDFI_BANNER);
    // Initialize client
    let endpoint = Endpoint::testnet();
    println!("Using endpoint: {}", endpoint);
    let timeout_ms = 10_000;
    let rpc_api = Arc::new(TonicRpcClient::new(&endpoint, timeout_ms));
    let keystore = FilesystemKeyStore::new("./keystore".into()).unwrap().into();
    // let seed = Word::new([Felt::new(1); 4]);
    // let client_rng = RpoRandomCoin::new(seed);
    let mut offer_rng = StdRng::seed_from_u64(43);
    let mut uuid_rng = StdRng::try_from_os_rng().unwrap();

    let mut client = ClientBuilder::new()
        //.rng(Box::new(client_rng))
        .rpc(rpc_api)
        .authenticator(keystore)
        .in_debug_mode(true.into())
        .build()
        .await?;

    let sync_summary = client.sync_state().await.unwrap();
    println!("Latest block: {}", sync_summary.block_num);

    let client_account = create_basic_account(&mut client).await?;
    let _ = client.sync_state().await.unwrap();
    let desk_account = deploy_book_account(&mut client).await?;
    let _ = client.sync_state().await.unwrap();

    let desk_account_address = Address::from(AccountIdAddress::new(
        desk_account.id(),
        AddressInterface::Unspecified
    )).to_bech32(NetworkId::Testnet);

    let client_account_address = Address::from(AccountIdAddress::new(
        client_account.id(),
        AddressInterface::Unspecified
    )).to_bech32(NetworkId::Testnet);
    
    println!("client_account: {}", client_account_address);
    println!("desk_account: {}", desk_account_address);

    let sync_summary = client.sync_state().await.unwrap();
    println!("Latest block: {}", sync_summary.block_num);

    let sync_summary = client.sync_state().await.unwrap();
    println!("Latest block: {}", sync_summary.block_num);

    for i in 1..=1000 {
        println!("\n=== Iteration {} ===", i);
        
        let note = create_note(&mut client, &mut offer_rng, &mut uuid_rng,&client_account, &desk_account).await?;
        let sync_summary = client.sync_state().await.unwrap();
        println!("Latest block: {}", sync_summary.block_num);

        consume_note(&mut client, &note, &desk_account).await?;
        let sync_summary = client.sync_state().await.unwrap();
        println!("Latest block: {}", sync_summary.block_num);


        let account_record = client.try_get_account(desk_account.id()).await?;
        let StorageSlot::Map(book) = &account_record.account().storage().slots()[1] else { todo!() };
        let StorageSlot::Value(head) = &account_record.account().storage().slots()[2] else { todo!() };
        println!("<<< Book");
        println!("Head: {}", head.as_elements()[0].as_int());
        for entry in book.entries().into_iter() {
            println!("{:?}", entry)
        }
        println!(">>> Book");
    }

    Ok(())
}
