import BN from "bn.js";
import * as web3 from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import type { Tubbly } from "../target/types/tubbly";

// Configure the client to use the local cluster
anchor.setProvider(anchor.AnchorProvider.env());

const program = anchor.workspace.Tubbly as anchor.Program<Tubbly>;

// client.ts - Solana Playground test file

// First, let's check if the program is properly loaded
console.log("Program ID:", program.programId.toString());
console.log("Program methods:", Object.keys(program.methods));

// Test submit function
console.log("=== Testing Tubbly Contract ===");
console.log("Wallet:", program.provider.publicKey.toString());

const testSubmit = async () => {
  try {
    // Generate unique request ID - use smaller number for testing
    const reqId = new anchor.BN(12345);  // Simple number for testing
    const amount = new anchor.BN(1_000_000_000); // 1 SOL in lamports
    
    console.log("Submitting request...");
    console.log("Request ID:", reqId.toString());
    console.log("Amount:", amount.toString(), "lamports");
    
    // Derive the request PDA first
    const [requestPDA] = await web3.PublicKey.findProgramAddress(
      [
        Buffer.from("request"),
        reqId.toArrayLike(Buffer, 'le', 16)
      ],
      program.programId
    );
    
    console.log("Request PDA:", requestPDA.toString());
    
    // Call submit with explicit PDA
    const tx = await program.methods
      .submit(reqId, amount)
      .accounts({
        request: requestPDA,  // Explicitly provide the PDA
        user: program.provider.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
    
    console.log("✅ Submit successful!");
    console.log("Transaction:", tx);
    console.log(`View on explorer: https://explorer.solana.com/tx/${tx}?cluster=devnet`);
    
    // Fetch and display the request data
    const requestAccount = await program.account.request.fetch(requestPDA);
    console.log("\nRequest Data:");
    console.log("- Caller:", requestAccount.caller.toString());
    console.log("- Balance:", requestAccount.balance.toString());
    console.log("- Is Active:", requestAccount.isActive);
    
    return reqId; // Return for use in confirm
    
  } catch (error) {
    console.error("❌ Error:", error);
    console.log("\nDetailed error:", error.logs || error.message);
  }
};

const testConfirm = async (reqId) => {
  try {
    console.log("\n=== Testing Confirm ===");
    console.log("Confirming request ID:", reqId.toString());
    
    // Derive PDAs
    const [statePDA] = await web3.PublicKey.findProgramAddress(
      [Buffer.from("state")],
      program.programId
    );
    
    const [requestPDA] = await web3.PublicKey.findProgramAddress(
      [
        Buffer.from("request"),
        reqId.toArrayLike(Buffer, 'le', 16)
      ],
      program.programId
    );
    
    // Get the request to find the caller
    const request = await program.account.request.fetch(requestPDA);
    
    const [userAccountPDA] = await web3.PublicKey.findProgramAddress(
      [
        Buffer.from("user"),
        request.caller.toBuffer()
      ],
      program.programId
    );
    
    console.log("State PDA:", statePDA.toString());
    console.log("Request PDA:", requestPDA.toString());
    console.log("User Account PDA:", userAccountPDA.toString());
    
    // Call confirm
    const tx = await program.methods
      .confirm(reqId)
      .accounts({
        state: statePDA,
        request: requestPDA,
        userAccount: userAccountPDA,
        owner: program.provider.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
    
    console.log("✅ Confirm successful!");
    console.log("Transaction:", tx);
    
    // Check updated user balance
    const userAccount = await program.account.userAccount.fetch(userAccountPDA);
    console.log("\nUser Balance Updated:");
    console.log("- User:", request.caller.toString());
    console.log("- New Balance:", userAccount.balance.toString(), "lamports");
    
  } catch (error) {
    console.error("❌ Error:", error);
    console.log("\nNote: Only the owner can confirm requests!");
  }
};

const testBalance = async (userAddress) => {
  try {
    console.log("\n=== Checking Balance ===");
    
    const userPubkey = new web3.PublicKey(userAddress || program.provider.publicKey);
    
    const [userAccountPDA] = await web3.PublicKey.findProgramAddress(
      [
        Buffer.from("user"),
        userPubkey.toBuffer()
      ],
      program.programId
    );
    
    const userAccount = await program.account.userAccount.fetch(userAccountPDA);
    console.log("User:", userPubkey.toString());
    console.log("Balance:", userAccount.balance.toString(), "lamports");
    
  } catch (error) {
    console.log("User account not found (no balance yet)");
  }
};

// Run tests
const runTests = async () => {
  console.log("Starting tests...\n");
  
  // Test 1: Submit a request
  const reqId = await testSubmit();
  
  if (reqId) {
    // Wait a bit for transaction to finalize
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Test 2: Confirm the request (only works if you're the owner)
    await testConfirm(reqId);
    
    // Test 3: Check balance
    await testBalance(program.provider.publicKey.toString());
  }
};

// Execute tests
runTests();