import type { Principal } from '@dfinity/principal';
export interface ColonyInfo {
  'player_count' : bigint,
  'taxes_percent' : number,
  'canister_id' : Principal,
  'generation' : number,
  'coffers' : Inventory,
  'expeditions_count' : bigint,
  'rewards_per_second' : Array<[Resources, number]>,
}
export interface ExpeditionState {
  'id' : bigint,
  'members' : Array<Principal>,
  'step' : ExpeditionStep,
  'resources_required' : Array<[Resources, bigint]>,
  'proposed_at' : bigint,
  'proposed_by' : Principal,
  'resources_pool' : Inventory,
}
export type ExpeditionStep = { 'Started' : Principal } |
  { 'Starting' : bigint } |
  { 'Done' : null } |
  { 'Ready' : null } |
  { 'Proposed' : null };
export interface Inventory {
  'contents' : Array<[Resources, bigint]>,
  'size' : number,
}
export type Resources = { 'Stone' : null } |
  { 'Food' : null } |
  { 'Gold' : null } |
  { 'Wood' : null } |
  { 'Water' : null };
export type Result = { 'Ok' : null } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : Array<[Resources, bigint]> } |
  { 'Err' : string };
export interface _SERVICE {
  'addPlayerToWorld' : () => Promise<Result>,
  'demoAddResourcesToExpedition' : () => Promise<Result>,
  'expeditionNext' : (arg_0: bigint) => Promise<Result>,
  'getColonyInfo' : () => Promise<ColonyInfo>,
  'getExpeditions' : () => Promise<Array<[bigint, ExpeditionState]>>,
  'getPlayerInventory' : () => Promise<Inventory>,
  'getRemoteColonies' : () => Promise<Array<Principal>>,
  'getUnclaimedWork' : () => Promise<Result_1>,
  'greet' : (arg_0: string) => Promise<string>,
  'isPlayerHere' : () => Promise<boolean>,
  'joinExpedition' : (arg_0: bigint) => Promise<Result>,
  'startExpedition' : () => Promise<Result>,
  'startWork' : () => Promise<Result>,
  'stopWork' : () => Promise<Result>,
  'wasm_sha256' : () => Promise<string>,
}
