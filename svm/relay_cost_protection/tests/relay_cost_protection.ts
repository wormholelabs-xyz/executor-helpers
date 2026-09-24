import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { RelayCostProtection } from "../target/types/relay_cost_protection";

describe("relay_cost_protection", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace
    .relayCostProtection as Program<RelayCostProtection>;

  const ACCOUNT_BALANCE = 1000000; // Transfer exactly 1,000,000 lamports to test account

  const testCases = [
    { minBalance: 500000, shouldSucceed: true },
    { minBalance: 1000000, shouldSucceed: true },
    { minBalance: 10000000, shouldSucceed: false },
  ];

  testCases.forEach(({ minBalance, shouldSucceed }) => {
    it(`Check balance with minBalance=${minBalance} (should ${shouldSucceed ? "succeed" : "fail"})`, async () => {
      const provider = anchor.AnchorProvider.env();
      const testKeypair = anchor.web3.Keypair.generate();

      const transferIx = anchor.web3.SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: testKeypair.publicKey,
        lamports: BigInt(ACCOUNT_BALANCE),
      });

      if (shouldSucceed) {
        const tx = await program.methods
          .checkBalance(new BN(minBalance))
          .accounts({
            account: testKeypair.publicKey,
          })
          .preInstructions([transferIx])
          .rpc();

        console.log(`Transaction signature: ${tx}`);
      } else {
        try {
          await program.methods
            .checkBalance(new BN(minBalance))
            .accounts({
              account: testKeypair.publicKey,
            })
            .preInstructions([transferIx])
            .rpc();

          throw new Error("Expected transaction to fail but it succeeded");
        } catch (error) {
          // Verify it's the correct error (InsufficientBalance)
          if (error.toString().includes("InsufficientBalance")) {
            console.log(`Transaction correctly failed with InsufficientBalance error`);
          } else {
            throw error;
          }
        }
      }
    });
  });
});
