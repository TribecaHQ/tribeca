use crate::*;

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, Lock<'info>>,
    amount: u64,
    duration: i64,
) -> Result<()> {
    invariant!(
        !ctx.accounts.locker.params.whitelist_enabled,
        MustProvideWhitelist
    );
    ctx.accounts.lock(amount, duration)
}
