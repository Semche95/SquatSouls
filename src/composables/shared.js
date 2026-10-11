import { inject, provide } from "vue";

// État créé par App et partagé avec ses descendants : un état de module survivrait d'un montage à l'autre.
export function defineShared(factory) {
  const key = Symbol();
  const provideShared = (...deps) => {
    const value = factory(...deps);
    provide(key, value);
    return value;
  };
  return [provideShared, () => inject(key)];
}
