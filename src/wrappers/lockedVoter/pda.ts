import { utils } from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";

import { TRIBECA_ADDRESSES } from "../../constants";

export const findLockerAddress = async (
  base: PublicKey
): Promise<[PublicKey, number]> => {
  return await PublicKey.findProgramAddress(
    [utils.bytes.utf8.encode("Locker"), base.toBuffer()],
    TRIBECA_ADDRESSES.LockedVoter
  );
};

export const findEscrowAddress = async (
  locker: PublicKey,
  authority: PublicKey
): Promise<[PublicKey, number]> => {
  return await PublicKey.findProgramAddress(
    [
      utils.bytes.utf8.encode("Escrow"),
      locker.toBuffer(),
      authority.toBuffer(),
    ],
    TRIBECA_ADDRESSES.LockedVoter
  );
};

export const findWhitelistAddress = async (
  locker: PublicKey,
  programId: PublicKey
): Promise<[PublicKey, number]> => {
  return await PublicKey.findProgramAddress(
    [
      utils.bytes.utf8.encode("LockerWhitelistEntry"),
      locker.toBuffer(),
      programId.toBuffer(),
    ],
    TRIBECA_ADDRESSES.LockedVoter
  );
};
