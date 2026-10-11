import { inject, provide } from 'vue';

// State created by App and shared with its descendants: module state would survive from one mount to the next.
export function defineShared(factory) {
  const key = Symbol();
  const provideShared = (...deps) => {
    const value = factory(...deps);
    provide(key, value);
    return value;
  };
  return [provideShared, () => inject(key)];
}
