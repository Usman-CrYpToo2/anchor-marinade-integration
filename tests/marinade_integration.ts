import * as anchor from "@coral-xyz/anchor";
import { AnchorError, BN, Program } from "@coral-xyz/anchor";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { assert } from "chai";
import { MarinadeIntegration } from "../target/types/marinade_integration";
import {
  LIQ_POOL_MSOL_LEG,
  LIQ_POOL_MSOL_LEG_AUTHORITY,
  LIQ_POOL_SOL_LEG_PDA,
  MARINADE_PROGRAM_ID,
  MSOL_MINT,
  MSOL_MINT_AUTHORITY,
  RESERVE_PDA,
  STATE,
  TREASURY_MSOL_ACCOUNT,
} from "./constants";

// Runs against a local validator that clones Marinade from mainnet-beta
// (see [test.validator] in Anchor.toml).
describe("marinade_integration", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace
    .marinadeIntegration as Program<MarinadeIntegration>;
  const connection = provider.connection;

  const user = Keypair.generate();
  const userMsol = anchor.utils.token.associatedAddress({
    mint: MSOL_MINT,
    owner: user.publicKey,
  });
  const ticketId = new BN(1);
  const [ticket] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("ticket"),
      STATE.toBuffer(),
      user.publicKey.toBuffer(),
      ticketId.toArrayLike(Buffer, "le", 8),
    ],
    program.programId
  );

  const DEPOSIT = new BN(2 * LAMPORTS_PER_SOL);
  const LIQUID_UNSTAKE = new BN(LAMPORTS_PER_SOL / 2);
  const ORDER_UNSTAKE = new BN(LAMPORTS_PER_SOL / 2);

  const msolBalance = async (): Promise<bigint> =>
    BigInt((await connection.getTokenAccountBalance(userMsol)).value.amount);

  const airdrop = async (to: PublicKey, sol: number) => {
    const signature = await connection.requestAirdrop(
      to,
      sol * LAMPORTS_PER_SOL
    );
    const blockhash = await connection.getLatestBlockhash();
    await connection.confirmTransaction(
      { signature, ...blockhash },
      "confirmed"
    );
  };

  const expectFailure = async (tx: Promise<unknown>, errorCode?: string) => {
    try {
      await tx;
    } catch (err) {
      if (errorCode) {
        assert.instanceOf(err, AnchorError);
        assert.equal((err as AnchorError).error.errorCode.code, errorCode);
      }
      return;
    }
    assert.fail("transaction should have failed");
  };

  const depositAccounts = () => ({
    liqPoolSolLegPda: LIQ_POOL_SOL_LEG_PDA,
    liqPoolMsolLeg: LIQ_POOL_MSOL_LEG,
    liqPoolMsolLegAuthority: LIQ_POOL_MSOL_LEG_AUTHORITY,
    reservePda: RESERVE_PDA,
    transferFrom: user.publicKey,
    msolMintAuthority: MSOL_MINT_AUTHORITY,
  });

  before(async () => {
    await airdrop(provider.wallet.publicKey, 10);
    await airdrop(user.publicKey, 10);
  });

  it("deposit rejects a zero amount", async () => {
    await expectFailure(
      program.methods
        .deposit(new BN(0))
        .accountsPartial(depositAccounts())
        .signers([user])
        .rpc(),
      "InvalidAmount"
    );
  });

  it("deposit stakes SOL and mints mSOL to the caller", async () => {
    await program.methods
      .deposit(DEPOSIT)
      .accountsPartial(depositAccounts())
      .signers([user])
      .rpc();

    const msol = await msolBalance();
    assert.isTrue(msol > 0n, "mSOL minted");
    // mSOL accrues staking rewards, so it trades above 1 SOL: 2 SOL buys < 2 mSOL.
    assert.isTrue(msol < BigInt(DEPOSIT.toString()), "mSOL priced above SOL");
  });

  it("liquid_unstake burns mSOL and pays SOL immediately", async () => {
    const msolBefore = await msolBalance();
    const solBefore = await connection.getBalance(user.publicKey);

    await program.methods
      .liquidUnstake(LIQUID_UNSTAKE)
      .accountsPartial({
        liqPoolSolLegPda: LIQ_POOL_SOL_LEG_PDA,
        liqPoolMsolLeg: LIQ_POOL_MSOL_LEG,
        treasuryMsolAccount: TREASURY_MSOL_ACCOUNT,
        getMsolFromAuthority: user.publicKey,
        transferSolTo: user.publicKey,
      })
      .signers([user])
      .rpc();

    assert.equal(
      await msolBalance(),
      msolBefore - BigInt(LIQUID_UNSTAKE.toString())
    );
    assert.isAbove(
      await connection.getBalance(user.publicKey),
      solBefore,
      "SOL received"
    );
  });

  it("liquid_unstake rejects more mSOL than the caller holds", async () => {
    const tooMuch = new BN((await msolBalance()).toString()).addn(1);
    await expectFailure(
      program.methods
        .liquidUnstake(tooMuch)
        .accountsPartial({
          liqPoolSolLegPda: LIQ_POOL_SOL_LEG_PDA,
          liqPoolMsolLeg: LIQ_POOL_MSOL_LEG,
          treasuryMsolAccount: TREASURY_MSOL_ACCOUNT,
          getMsolFromAuthority: user.publicKey,
          transferSolTo: user.publicKey,
        })
        .signers([user])
        .rpc(),
      "InsufficientMsol"
    );
  });

  const orderUnstake = () =>
    program.methods
      .orderUnstake(ORDER_UNSTAKE, ticketId)
      .accountsPartial({
        burnMsolAuthority: user.publicKey,
        payer: user.publicKey,
        newTicketAccount: ticket,
      })
      .signers([user])
      .rpc();

  it("order_unstake burns mSOL and opens a Marinade ticket for the caller", async () => {
    const msolBefore = await msolBalance();
    await orderUnstake();

    assert.equal(
      await msolBalance(),
      msolBefore - BigInt(ORDER_UNSTAKE.toString())
    );

    const info = await connection.getAccountInfo(ticket);
    assert.isNotNull(info);
    assert.isTrue(
      info!.owner.equals(MARINADE_PROGRAM_ID),
      "ticket owned by Marinade"
    );

    // TicketAccountData: discriminator(8) | state(32) | beneficiary(32) | lamports(8) | created_epoch(8)
    const data = info!.data;
    assert.isTrue(new PublicKey(data.subarray(8, 40)).equals(STATE));
    assert.isTrue(new PublicKey(data.subarray(40, 72)).equals(user.publicKey));
    const lamports = data.readBigUInt64LE(72);
    assert.isTrue(
      lamports >= BigInt(ORDER_UNSTAKE.toString()),
      "ticket worth at least the mSOL burned"
    );
  });

  it("order_unstake rejects reuse of a ticket ID", async () => {
    await expectFailure(orderUnstake());
  });

  it("claim rejects a ticket before it matures", async () => {
    const solBefore = await connection.getBalance(user.publicKey);

    await expectFailure(
      program.methods
        .claim(ticketId)
        .accountsPartial({
          reservePda: RESERVE_PDA,
          ticketAccount: ticket,
          transferSolTo: user.publicKey,
        })
        .rpc()
    );

    assert.equal(await connection.getBalance(user.publicKey), solBefore);
    assert.isNotNull(
      await connection.getAccountInfo(ticket),
      "ticket still open"
    );
  });
});
