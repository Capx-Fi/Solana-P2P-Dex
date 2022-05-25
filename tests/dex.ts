import assert from "assert";
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import * as spl from "@solana/spl-token";
import { Dex } from "../target/types/dex";
import { util } from "chai";
import { min } from "bn.js";

const {
  Connection,
  TransactionInstruction,
  Transaction,
  sendAndConfirmTransaction,
  PublicKey,
  SystemProgram,
} = anchor.web3;


describe("dex", async () => {
  // Configure the client to use the local cluster.
  const program = (await anchor.workspace.Dex) as Program<Dex>;
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);
  const userAccount = anchor.web3.Keypair.generate();
  let numerator = 24;
  let denominator = 59;
  let randomPubkey = anchor.web3.Keypair.generate();
  let mint: anchor.web3.Keypair;
  let mint2: anchor.web3.Keypair;
  let sender_token1: anchor.web3.PublicKey;
  let sender_token2: anchor.web3.PublicKey;

  let pda_1 : anchor.web3.Keypair;
  let userPDA;
  let bump_user;
  let user2PDA;
  let bump2_user;
  let vaultPDA;
  let bump_vault;
  let orderVaultPDA;
  let bump_ovault;

  

  it("Deploy SPL Tokens",async()=>{

    mint = anchor.web3.Keypair.generate();
    mint2 = anchor.web3.Keypair.generate();

    let create_mint_tx = new Transaction().add(
      // create mint account
      SystemProgram.createAccount({
        fromPubkey: provider.wallet.publicKey,
        newAccountPubkey: mint.publicKey,
        space: spl.MintLayout.span,
        lamports: await spl.getMinimumBalanceForRentExemptMint(program.provider.connection),
        programId: spl.TOKEN_PROGRAM_ID,
      }),
      // init mint account
      spl.createInitializeMintInstruction(mint.publicKey, 6, provider.wallet.publicKey, provider.wallet.publicKey, spl.TOKEN_PROGRAM_ID)
    );
    
    let create_mint_tx2 = new Transaction().add(
      // create mint account
      SystemProgram.createAccount({
        fromPubkey: provider.wallet.publicKey,
        newAccountPubkey: mint2.publicKey,
        space: spl.MintLayout.span,
        lamports: await spl.getMinimumBalanceForRentExemptMint(program.provider.connection),
        programId: spl.TOKEN_PROGRAM_ID,
      }),
      // init mint account
      spl.createInitializeMintInstruction(mint2.publicKey, 6, provider.wallet.publicKey, provider.wallet.publicKey, spl.TOKEN_PROGRAM_ID)
    );

    await program.provider.sendAndConfirm(create_mint_tx, [mint]);
    await program.provider.sendAndConfirm(create_mint_tx2, [mint2]);

    sender_token2 = await spl.getAssociatedTokenAddress(mint2.publicKey, provider.wallet.publicKey, false, spl.TOKEN_PROGRAM_ID, spl.ASSOCIATED_TOKEN_PROGRAM_ID)
    sender_token1 = await spl.getAssociatedTokenAddress(mint.publicKey, provider.wallet.publicKey, false, spl.TOKEN_PROGRAM_ID, spl.ASSOCIATED_TOKEN_PROGRAM_ID)
    
    let create_sender_token2_tx = new Transaction().add(
      // init mint account
      spl.createAssociatedTokenAccountInstruction(
        provider.wallet.publicKey, sender_token2, provider.wallet.publicKey, mint2.publicKey, spl.TOKEN_PROGRAM_ID, spl.ASSOCIATED_TOKEN_PROGRAM_ID
      )
    );
    let create_sender_token1_tx = new Transaction().add(
      // init mint account
      spl.createAssociatedTokenAccountInstruction(
        provider.wallet.publicKey, sender_token1, provider.wallet.publicKey, mint.publicKey, spl.TOKEN_PROGRAM_ID, spl.ASSOCIATED_TOKEN_PROGRAM_ID
      )
    );

    await program.provider.sendAndConfirm(create_sender_token2_tx);
    await program.provider.sendAndConfirm(create_sender_token1_tx);

    await console.log(mint.publicKey.toBase58());

  });


  it("Read Value of User PDA", async()=>{

    let mint_tokens_tx1 = new Transaction().add(
      spl.createMintToInstruction(// always TOKEN_PROGRAM_ID
        mint.publicKey, // mint
        sender_token1, // receiver (sholud be a token account)
        provider.wallet.publicKey, // mint authority
        3e6,
        [], // only multisig account will use. leave it empty now.
        spl.TOKEN_PROGRAM_ID,  // amount. if your decimals is 8, you mint 10^8 for 1 token.
      )
    );

    let mint_tokens_tx2 = new Transaction().add(
      spl.createMintToInstruction(// always TOKEN_PROGRAM_ID
        mint2.publicKey, // mint
        sender_token2, // receiver (sholud be a token account)
        provider.wallet.publicKey, // mint authority
        2e6,
        [], // only multisig account will use. leave it empty now.
        spl.TOKEN_PROGRAM_ID,  // amount. if your decimals is 8, you mint 10^8 for 1 token.
      )
    );

    await program.provider.sendAndConfirm(mint_tokens_tx1);
    await program.provider.sendAndConfirm(mint_tokens_tx2);

    await console.log(provider.wallet.publicKey)
     
    console.log("token1 balance in ATA: ", await program.provider.connection.getTokenAccountBalance(sender_token1));
    
    console.log("token2 balance in ATA: ", await program.provider.connection.getTokenAccountBalance(sender_token2));
    

  });


  it("Deposit token",async()=>{
    
    console.log("token1 balance in ATA: ", await program.provider.connection.getTokenAccountBalance(sender_token1));
    console.log("token2 balance in ATA: ", await program.provider.connection.getTokenAccountBalance(sender_token2));

    [vaultPDA, bump_vault] = await PublicKey.findProgramAddress(
          [
            anchor.utils.bytes.utf8.encode("user-vault"),
            mint.publicKey.toBuffer(),
            provider.wallet.publicKey.toBuffer()
      
          ],
          program.programId
        );

    const tx = await program.methods.depositToken(new anchor.BN(3000)).accounts({
          user: provider.wallet.publicKey,
          tokenMint : mint.publicKey,
          userVault : vaultPDA,
          tokenUserAta : sender_token1,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: spl.TOKEN_PROGRAM_ID
        }).rpc();

        console.log("token1 balance in ATA: ", await program.provider.connection.getTokenAccountBalance(sender_token1));
        console.log("token1 balance in ATA: ", await program.provider.connection.getTokenAccountBalance(sender_token2));
        console.log("vault balance in ATA: ", await program.provider.connection.getTokenAccountBalance(vaultPDA));

  });

  it("Deposit token 2",async()=>{
    
    console.log("token1 balance in ATA: ", await program.provider.connection.getTokenAccountBalance(sender_token1));
    console.log("token2 balance in ATA: ", await program.provider.connection.getTokenAccountBalance(sender_token2));

    [vaultPDA, bump_vault] = await PublicKey.findProgramAddress(
          [
            anchor.utils.bytes.utf8.encode("user-vault"),
            mint.publicKey.toBuffer(),
            provider.wallet.publicKey.toBuffer()
      
          ],
          program.programId
        );

    const tx = await program.methods.depositToken(new anchor.BN(3000)).accounts({
          user: provider.wallet.publicKey,
          tokenMint : mint.publicKey,
          userVault : vaultPDA,
          tokenUserAta : sender_token1,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: spl.TOKEN_PROGRAM_ID
        }).rpc();

        console.log("token1 balance in ATA: ", await program.provider.connection.getTokenAccountBalance(sender_token1));
        console.log("token1 balance in ATA: ", await program.provider.connection.getTokenAccountBalance(sender_token2));
        console.log("vault balance in ATA: ", await program.provider.connection.getTokenAccountBalance(vaultPDA));

  });

  it("Withdraw token",async()=>{
    
    
    [vaultPDA, bump_vault] = await PublicKey.findProgramAddress(
          [
            anchor.utils.bytes.utf8.encode("user-vault"),
            mint.publicKey.toBuffer(),
            provider.wallet.publicKey.toBuffer()
      
          ],
          program.programId
        );

    const tx = await program.methods.withdrawToken(bump_vault,new anchor.BN(2000)).accounts({
          user: provider.wallet.publicKey,
          tokenMint : mint.publicKey,
          userVault : vaultPDA,
          tokenUserAta : sender_token1,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: spl.TOKEN_PROGRAM_ID
        }).rpc();

        console.log("token1 balance in ATA: ", await program.provider.connection.getTokenAccountBalance(sender_token1));
        console.log("token1 balance in ATA: ", await program.provider.connection.getTokenAccountBalance(sender_token2));
        console.log("vault balance in ATA: ", await program.provider.connection.getTokenAccountBalance(vaultPDA));

  });


  it("Order creation with Vault", async()=>{

    [userPDA, bump_user] = await PublicKey.findProgramAddress(
          [
            anchor.utils.bytes.utf8.encode("user-account"),
            randomPubkey.publicKey.toBuffer(),
            provider.wallet.publicKey.toBuffer()
      
          ],
          program.programId
        );
    [orderVaultPDA, bump_ovault] = await PublicKey.findProgramAddress(
          [
            anchor.utils.bytes.utf8.encode("order-vault"),
            randomPubkey.publicKey.toBuffer(),
            provider.wallet.publicKey.toBuffer()
      
          ],
          program.programId
        );

      const tx = await program.methods.initOrder(bump_vault,randomPubkey.publicKey,new anchor.BN(100),new anchor.BN(50),new anchor.BN(1653552655)).accounts({
      user: provider.wallet.publicKey,
      userAccount: userPDA,
      token1Mint : mint.publicKey,
      token2Mint : mint2.publicKey,
      userVault : vaultPDA,
      orderVault : orderVaultPDA,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: spl.TOKEN_PROGRAM_ID
    }).rpc();


    console.log("vault balance in ATA: ", await program.provider.connection.getTokenAccountBalance(vaultPDA));
    console.log("order vault balance in ATA: ", await program.provider.connection.getTokenAccountBalance(orderVaultPDA));

  });
  

});
