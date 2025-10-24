/**
 * Example client code for interacting with the ExchAInge Program
 *
 * Copy this to your backend and modify as needed
 */

import { Connection, PublicKey, Keypair } from '@solana/web3.js';
import { Program, AnchorProvider, Wallet } from '@coral-xyz/anchor';
import IDL from './target/idl/exchainge_program.json';

// Program ID (Testnet)
const PROGRAM_ID = new PublicKey('2yvGQ26fz2mvPnxDa2wcTf5Y88hr9sTSJpiZdFqMyQ4L');

// Initialize connection and provider
function getProvider(rpcUrl: string, walletKeypair: Keypair) {
  const connection = new Connection(rpcUrl, 'confirmed');
  const wallet = new Wallet(walletKeypair);
  return new AnchorProvider(connection, wallet, { commitment: 'confirmed' });
}

// Initialize program
function getProgram(provider: AnchorProvider) {
  return new Program(IDL as any, provider);
}

/**
 * Register a new key-hash mapping on-chain
 */
export async function registerData(
  rpcUrl: string,
  walletKeypair: Keypair,
  internalKey: string,
  datasetHash: string
) {
  const provider = getProvider(rpcUrl, walletKeypair);
  const program = getProgram(provider);

  // Generate a new keypair for the registry account
  const registryKeypair = Keypair.generate();

  try {
    const tx = await program.methods
      .register(internalKey, datasetHash)
      .accounts({
        registry: registryKeypair.publicKey,
        owner: walletKeypair.publicKey,
        systemProgram: new PublicKey('11111111111111111111111111111111'),
      })
      .signers([registryKeypair])
      .rpc();

    console.log('✅ Register transaction:', tx);
    console.log('📝 Registry account:', registryKeypair.publicKey.toBase58());

    return {
      signature: tx,
      registryAddress: registryKeypair.publicKey.toBase58(),
    };
  } catch (error) {
    console.error('❌ Register failed:', error);
    throw error;
  }
}

/**
 * Update dataset hash for existing registry
 */
export async function updateData(
  rpcUrl: string,
  walletKeypair: Keypair,
  registryAddress: string,
  newDatasetHash: string
) {
  const provider = getProvider(rpcUrl, walletKeypair);
  const program = getProgram(provider);

  try {
    const tx = await program.methods
      .update(newDatasetHash)
      .accounts({
        registry: new PublicKey(registryAddress),
        owner: walletKeypair.publicKey,
      })
      .rpc();

    console.log('✅ Update transaction:', tx);
    return { signature: tx };
  } catch (error) {
    console.error('❌ Update failed:', error);
    throw error;
  }
}

/**
 * Fetch registry data from on-chain
 */
export async function fetchRegistry(
  rpcUrl: string,
  registryAddress: string
) {
  const connection = new Connection(rpcUrl, 'confirmed');
  const wallet = new Wallet(Keypair.generate()); // Read-only, any keypair works
  const provider = new AnchorProvider(connection, wallet, { commitment: 'confirmed' });
  const program = getProgram(provider);

  try {
    const registryData = await program.account.dataRegistry.fetch(
      new PublicKey(registryAddress)
    );

    console.log('📖 Registry data:', {
      owner: registryData.owner.toBase58(),
      internalKey: registryData.internalKey,
      datasetHash: registryData.datasetHash,
      createdAt: new Date(registryData.createdAt.toNumber() * 1000).toISOString(),
    });

    return registryData;
  } catch (error) {
    console.error('❌ Fetch failed:', error);
    throw error;
  }
}

/**
 * Close registry and reclaim rent
 */
export async function closeRegistry(
  rpcUrl: string,
  walletKeypair: Keypair,
  registryAddress: string
) {
  const provider = getProvider(rpcUrl, walletKeypair);
  const program = getProgram(provider);

  try {
    const tx = await program.methods
      .close()
      .accounts({
        registry: new PublicKey(registryAddress),
        owner: walletKeypair.publicKey,
      })
      .rpc();

    console.log('✅ Close transaction:', tx);
    return { signature: tx };
  } catch (error) {
    console.error('❌ Close failed:', error);
    throw error;
  }
}

// Example usage (for testing)
async function example() {
  const RPC_URL = process.env.NEXT_PUBLIC_SOLANA_RPC_URL || 'https://api.testnet.solana.com';

  // Load your wallet (in production, use secure key management!)
  const wallet = Keypair.fromSecretKey(
    Buffer.from(JSON.parse(process.env.SOLANA_WALLET_PRIVATE_KEY!))
  );

  // Register new data
  const { registryAddress } = await registerData(
    RPC_URL,
    wallet,
    'user123_file456',
    'bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi' // IPFS hash example
  );

  // Fetch the data
  await fetchRegistry(RPC_URL, registryAddress);

  // Update the hash
  await updateData(
    RPC_URL,
    wallet,
    registryAddress,
    'bafybeih2w5hvuy2jkgkwfq6gvqpqxccxtq6ym46a2m5qp6h7z6vq6sdzm4'
  );

  // Close and reclaim rent
  await closeRegistry(RPC_URL, wallet, registryAddress);
}
