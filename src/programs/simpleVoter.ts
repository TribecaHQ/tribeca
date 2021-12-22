import type { AnchorTypes } from "@saberhq/anchor-contrib";

import type { SimpleVoterIDL } from "../idls/simple_voter";

export * from "../idls/simple_voter";

export type SimpleVoterTypes = AnchorTypes<
  SimpleVoterIDL,
  {
    electorate: ElectorateData;
    tokenRecord: TokenRecordData;
  }
>;

type Accounts = SimpleVoterTypes["Accounts"];
export type ElectorateData = Accounts["Electorate"];
export type TokenRecordData = Accounts["TokenRecord"];

export type SimpleVoterProgram = SimpleVoterTypes["Program"];
