
Simple implementation of an order book as a double-linked list stored in a public Miden account. Entries are randomly generated for now.

To run
> cargo run --release

There seems to be an issue at iteration 110.

```
=== Iteration 110 ===
one or more warnings were emitted
one or more warnings were emitted

BTCUSD Market
Offer amount: 30 price: 141881
View transaction on MidenScan: https://testnet.midenscan.com/tx/0x48302004abe011ae82df1207301f9566e071968e854dbeb2b7d295768ac2e1c0
Latest block: 265888

thread 'main' panicked at src/main.rs:65:10:
called `Result::unwrap()` on an `Err` value: TransactionExecutorError(TransactionProgramExecutionFailed(FailedToExecuteProgram("stack underflow when restoring context")))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```