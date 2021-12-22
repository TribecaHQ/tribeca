import { utils } from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";

import { TRIBECA_ADDRESSES } from "../../constants";

export const findSimpleElectorateAddress = async (
  base: PublicKey
): Promise<[PublicKey, number]> => {
  return await PublicKey.findProgramAddress(
    [utils.bytes.utf8.encode("SimpleElectorate"), base.toBuffer()],
    TRIBECA_ADDRESSES.SimpleVoter
  );
};

export const findTokenRecordAddress = async (
  authorityKey: PublicKey,
  electorateKey: PublicKey
): Promise<[PublicKey, number]> => {
  return await PublicKey.findProgramAddress(
    [
      utils.bytes.utf8.encode("SimpleTokenRecord"),
      authorityKey.toBuffer(),
      electorateKey.toBuffer(),
    ],
    TRIBECA_ADDRESSES.SimpleVoter
  );
};
