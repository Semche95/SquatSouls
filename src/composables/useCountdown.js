import { computed, ref } from 'vue';

// Refresh rate of the countdowns.
const TICK_MS = 250;

// Countdown that can be held then resumed; `onEnd` runs when it reaches 0, right away for 0 s.
export function useCountdown() {
  const end = ref(0);
  const seconds = ref(0);
  // Milliseconds left while held, 0 otherwise.
  const heldMs = ref(0);
  const now = ref(0);
  let ticker;
  let onEnd;
  const msLeft = computed(() => heldMs.value || Math.max(0, end.value - now.value));
  const left = computed(() => Math.ceil(msLeft.value / 1000));
  // Share still to wait, from 1 to 0.
  const share = computed(() => (seconds.value ? msLeft.value / (seconds.value * 1000) : 0));
  const held = computed(() => !!heldMs.value);

  function run(ms) {
    heldMs.value = 0;
    now.value = Date.now();
    end.value = now.value + ms;
    ticker = setInterval(() => {
      now.value = Date.now();
      if (!msLeft.value) finish(onEnd);
    }, TICK_MS);
  }

  function finish(then) {
    stop();
    then?.();
  }

  function start(duration, then) {
    stop();
    if (!duration) return then?.();
    seconds.value = duration;
    onEnd = then;
    run(duration * 1000);
  }

  function stop() {
    clearInterval(ticker);
    end.value = 0;
    seconds.value = 0;
    heldMs.value = 0;
    onEnd = undefined;
  }

  function hold() {
    clearInterval(ticker);
    heldMs.value = msLeft.value;
  }

  function resume() {
    if (heldMs.value) run(heldMs.value);
  }

  // Gives the running countdown a new duration, as if it had started with it: the time already elapsed is kept.
  // Already past it, ends right away, running `onEnd` unless the new duration is 0.
  function resize(duration) {
    if (!seconds.value || duration === seconds.value) return;
    const ms = duration * 1000 - (seconds.value * 1000 - msLeft.value);
    if (ms <= 0) return finish(duration ? onEnd : undefined);
    seconds.value = duration;
    if (heldMs.value) heldMs.value = ms;
    else {
      clearInterval(ticker);
      run(ms);
    }
  }

  return { left, share, held, start, stop, hold, resume, resize };
}
