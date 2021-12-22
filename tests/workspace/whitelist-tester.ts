import type { AnchorTypes } from "@saberhq/anchor-contrib";

import type { WhitelistTesterIDL } from "../../src/idls/whitelist_tester";

export * from "../../src/idls/whitelist_tester";

export type WhitelistTesterTypes = AnchorTypes<WhitelistTesterIDL>;
export type WhitelistTesterProgram = WhitelistTesterTypes["Program"];
