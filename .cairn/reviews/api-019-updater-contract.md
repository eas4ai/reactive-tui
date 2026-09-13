# Updater dispatch contract choice

Observed: src/ui/mod.rs declares only pub trait Updater {}. Its documentation
promises registration and refresh updates, but there is no registration or
dispatch implementation. Source, examples and tests contain no in-repository
implementation or caller. This does not establish absence of downstream users.
The API-019 inventory requires real dispatch with ordering and ownership and
rejects leaving this as a marker or dropping updates.

Proposed migration: retain the Updater name and require an update method.
Applications register an owned updater and receive a request handle. Requests
wake that App; pending requests coalesce per updater with bounded storage.
Before rendering, the App invokes pending updaters in registration order on its
own thread. Requests made during a callback run on a later turn. Registration
removal or App exit drops the updater and makes its old handles inert. Callback
failures propagate through App cleanup. Updaters can change ordinary reactive
state read by their intended component; no parallel rendering path is added.

Compatibility consequence: downstream empty impl Updater for T {} blocks must
implement the new method. Adding a default empty callback would compile those
blocks but preserve precisely the successful no-op that API-019 forbids.
A source-compatible alternative is explicit callback registration while retaining
the empty trait; that needs an approved contract change because the trait itself
would remain a marker rather than become the advertised update interface.

Proposed acceptance before claiming completion: two updaters update distinct
painted component state; registration-order delivery; repeated requests coalesce;
callback reentry schedules a later turn; separate Apps remain isolated; dropped
registrations and closed App handles are inert; callback errors/unwinds restore
terminal state and release ownership. A disabled dispatch mutation must fail the
painted-state consumer, and the corrected implementation must pass. A downstream
consumer must demonstrate the documented trait migration.

No Updater production code has been changed. The specification requires an
explicit developer decision for a breaking public migration. This review makes
the choice and its consequence concrete before requesting that decision.
