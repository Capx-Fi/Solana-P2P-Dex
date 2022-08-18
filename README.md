Solana P2P Dex
====================================

The project provides a decentralized platform to trade tokens using a P2P dex on solana.

Deploying the Smart Contracts
================================

Solana Programs when deployed involves sending multiple transactions to store the program logic in an specific account. The program ID is known from before as the transactions are signed using a new keypair.

Implementation Details 
======================

### Libraries Used



* `AssociatedToken` - Used to determine the associated token account
* `{CloseAccount, Mint, Token, TokenAccount, Transfer}` - Token operations 

### Functions

* `deposit_token` - Deposit a specific type of token in a PDA.
* `withdraw_token` - Withdraw a specific type of token in a PDA.
* `init_order` - Initialize a new P2P Order.
* `cancel_order` - Cancel an initialized order.
* `accept_order` - Accept an existing order from the order book.