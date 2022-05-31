export const idlFactory = ({ IDL }) => {
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  const Resources = IDL.Variant({
    'Stone' : IDL.Null,
    'Food' : IDL.Null,
    'Gold' : IDL.Null,
    'Wood' : IDL.Null,
    'Water' : IDL.Null,
  });
  const Inventory = IDL.Record({
    'contents' : IDL.Vec(IDL.Tuple(Resources, IDL.Nat64)),
    'size' : IDL.Nat32,
  });
  const ColonyInfo = IDL.Record({
    'player_count' : IDL.Nat64,
    'taxes_percent' : IDL.Nat8,
    'canister_id' : IDL.Principal,
    'generation' : IDL.Nat8,
    'coffers' : Inventory,
    'expeditions_count' : IDL.Nat64,
    'rewards_per_second' : IDL.Vec(IDL.Tuple(Resources, IDL.Nat8)),
  });
  const ExpeditionStep = IDL.Variant({
    'Started' : IDL.Principal,
    'Starting' : IDL.Nat64,
    'Done' : IDL.Null,
    'Ready' : IDL.Null,
    'Proposed' : IDL.Null,
  });
  const ExpeditionState = IDL.Record({
    'id' : IDL.Nat64,
    'members' : IDL.Vec(IDL.Principal),
    'step' : ExpeditionStep,
    'resources_required' : IDL.Vec(IDL.Tuple(Resources, IDL.Nat64)),
    'proposed_at' : IDL.Nat64,
    'proposed_by' : IDL.Principal,
    'resources_pool' : Inventory,
  });
  const Result_1 = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Tuple(Resources, IDL.Nat64)),
    'Err' : IDL.Text,
  });
  return IDL.Service({
    'addPlayerToWorld' : IDL.Func([], [Result], []),
    'demoAddResourcesToExpedition' : IDL.Func([], [Result], []),
    'expeditionNext' : IDL.Func([IDL.Nat64], [Result], []),
    'getColonyInfo' : IDL.Func([], [ColonyInfo], ['query']),
    'getExpeditions' : IDL.Func(
        [],
        [IDL.Vec(IDL.Tuple(IDL.Nat64, ExpeditionState))],
        ['query'],
      ),
    'getPlayerInventory' : IDL.Func([], [Inventory], ['query']),
    'getRemoteColonies' : IDL.Func([], [IDL.Vec(IDL.Principal)], ['query']),
    'getUnclaimedWork' : IDL.Func([], [Result_1], ['query']),
    'greet' : IDL.Func([IDL.Text], [IDL.Text], ['query']),
    'isPlayerHere' : IDL.Func([], [IDL.Bool], ['query']),
    'joinExpedition' : IDL.Func([IDL.Nat64], [Result], []),
    'startExpedition' : IDL.Func([], [Result], []),
    'startWork' : IDL.Func([], [Result], []),
    'stopWork' : IDL.Func([], [Result], []),
    'wasm_sha256' : IDL.Func([], [IDL.Text], ['query']),
  });
};
export const init = ({ IDL }) => { return []; };
