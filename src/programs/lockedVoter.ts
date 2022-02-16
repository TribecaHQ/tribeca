import type { AnchorTypes } from "@saberhq/anchor-contrib";

import type { LockedVoterIDL } from "../idls/locked_voter";

export * from "../idls/locked_voter";

export type LockedVoterTypes = AnchorTypes<
  LockedVoterIDL,
  {
    locker: LockerData;
    escrow: EscrowData;
    lockerWhitelistEntry: LockerWhitelistEntryData;
  }
>;

type Accounts = LockedVoterTypes["Accounts"];
export type LockerData = Accounts["Locker"];
export type EscrowData = Accounts["Escrow"];
export type LockerWhitelistEntryData = Accounts["LockerWhitelistEntry"];

export type LockerParams = LockedVoterTypes["Defined"]["LockerParams"];

export type LockedVoterError = LockedVoterTypes["Error"];
export type LockedVoterProgram = LockedVoterTypes["Program"];
