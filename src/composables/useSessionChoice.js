import { watch } from 'vue';

// Choice made in Play that the session carries, such as the intensity or the limit: `current()` the one shown, which
// `apply` sets, `of(session)` the session's own. It shows the session's own while the session has a game, and is
// sent to the session by `command` when changed during it, `arg` naming it. A session resumed at launch brings the
// choice of an earlier launch: once it ends, however it ends, unless the choice changed meanwhile, `reset()` gives
// the one a launch gives. Returns `resumed(session)`, to call on the session read back at launch.
export function useSessionChoice({ session, call, reload }, { current, apply, of, command, arg, reset }) {
  const same = (a, b) => JSON.stringify(a ?? null) === JSON.stringify(b ?? null);
  watch(session, (s) => {
    if (s.gameId && !same(current(), of(s))) apply(of(s));
  });

  let inherited = false;
  watch(() => session.value.startedAt, (started) => {
    if (started || !inherited) return;
    inherited = false;
    reset();
  });

  watch(current, async (value) => {
    if (!session.value.gameId || same(value, of(session.value))) return;
    inherited = false;
    try {
      session.value = await call(command, { [arg]: value });
    } catch {
      await reload();
    }
  });

  return (s) => {
    inherited = !!s.gameId;
  };
}
