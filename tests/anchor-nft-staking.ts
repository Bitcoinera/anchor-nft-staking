import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { AnchorNftStaking } from "../target/types/anchor_nft_staking";
import { LootBoxes } from "../target/types/loot_boxes";
import { setupNft } from "./helpers/setup_nft";
import { PROGRAM_ID as METADATA_PROGRAM_ID } from '@metaplex-foundation/mpl-token-metadata';
import { expect } from "chai";
import {
  getOrCreateAssociatedTokenAccount,
  getAssociatedTokenAddress,
  getAccount,
  createMint,
  mintToChecked,
} from "@solana/spl-token";
import { BN } from "bn.js";

describe("anchor-nft-staking", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const wallet = anchor.workspace.AnchorNftStaking.provider.wallet;

  const program = anchor.workspace.AnchorNftStaking as Program<AnchorNftStaking>;
  const lootboxProgram = anchor.workspace.LootBoxes as Program<LootBoxes>;

  let delegatedAuthPda: anchor.web3.PublicKey;
  let stakeStatePda: anchor.web3.PublicKey;
  let nft: any;
  let mintAuth: anchor.web3.PublicKey;
  let mint: anchor.web3.PublicKey;
  let tokenAddress: anchor.web3.PublicKey;


  before(async () => {
    ({ nft, delegatedAuthPda, stakeStatePda, mint, mintAuth, tokenAddress } =
      await setupNft(program, wallet.payer));
  });

  it("Stake", async () => {
    const tx = await program.methods
      .stake()
      .accounts({
        nftTokenAccount: nft.tokenAddress,
        nftMint: nft.mintAddress,
        nftEdition: nft.masterEditionAddress,
        metadataProgram: METADATA_PROGRAM_ID,
      })
      .rpc();

      console.log( `View transaction: https://explorer.solana.com/tx/${tx}?cluster=devnet` );

    const account = await program.account.userStakeInfo.fetch(stakeStatePda);
    expect(account.stakeState === "Staked");
  });

  it("Redeems", async () => {
    const tx = await program.methods
      .redeem()
      .accounts({
        nftTokenAccount: nft.tokenAddress,
        stakeMint: mint,
        userStakeAta: tokenAddress,
      })
      .rpc();
    
      console.log(`View transaction: https://explorer.solana.com/tx/${tx}?cluster=devnet`);

    const account = await program.account.userStakeInfo.fetch(stakeStatePda);
    expect(account.stakeState === "Staked");
    const tokenAccount = await getAccount(provider.connection, tokenAddress);
    console.log(tokenAccount.amount);
  });

  it("Unstake", async () => {
    const tx = await program.methods
      .unstake()
      .accounts({
        nftTokenAccount: nft.tokenAddress,
        nftMint: nft.mintAddress,
        nftEdition: nft.masterEditionAddress,
        metadataProgram: METADATA_PROGRAM_ID,
        stakeMint: mint,
        userStakeAta: tokenAddress,
      })
      .rpc();
    
      console.log( `View transaction: https://explorer.solana.com/tx/${tx}?cluster=devnet` );

    const account = await program.account.userStakeInfo.fetch(stakeStatePda);
    expect(account.stakeState === "Unstaked");
    const tokenAccount = await getAccount(provider.connection, tokenAddress);
    console.log(tokenAccount.amount);
  });

  it("Chooses a random lootbox",async () => {

    const [stakeAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [wallet.publicKey.toBuffer(), nft.tokenAddress.toBuffer()],
      program.programId
    );

    await lootboxProgram.methods
      .openLootbox(new BN(10))
      .accounts({
        stakeMint: mint,
        userStakeAta: tokenAddress,
        stakeState: stakeAccount,
      })
      .rpc();

    const [address] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("lootbox"), wallet.publicKey.toBuffer()],
      lootboxProgram.programId
    );
    const pointer = await lootboxProgram.account.lootboxPointer.fetch(address);

    expect(pointer.isInitialized == true);
    expect(pointer.isClaimed == false);

  })


  it("Claim the selected gear", async () => {
    
    const [lootboxPointerAddress] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("lootbox"), wallet.publicKey.toBuffer()],
        lootboxProgram.programId
      );

    const pointer = await lootboxProgram.account.lootboxPointer.fetch(
      lootboxPointerAddress
    );
    console.log(pointer.mint.toBase58());
    
    const gearAta = await getAssociatedTokenAddress(
      pointer.mint,
      wallet.publicKey
    );
    console.log(gearAta.toBase58())

    await lootboxProgram.methods
      .claimLootbox()
      .accounts({
        lootboxPointer: lootboxPointerAddress,
        gearMint: pointer.mint,
        userGearAta: gearAta,
      })
      .rpc();

    const gearAccount = await getAccount(provider.connection, gearAta);
    expect(Number(gearAccount.amount)).to.equal(1);
  });

});
