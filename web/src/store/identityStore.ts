/**
 * Identity store — React state for the currently loaded identity.
 *
 * This is the ONLY place the npub is stored in JS state.
 * Secret material (mnemonic, seed bytes) never enters this store.
 */

import { create } from "zustand";
import { persist } from "zustand/middleware";

interface IdentityState {
  /** Current identity's npub (bech32). Null = not onboarded. */
  npub: string | null;
  /** Account index used for derivation. */
  accountIndex: number;
  /** Set the active identity's public info. */
  setIdentity: (npub: string, accountIndex?: number) => void;
  /** Clear the identity (logout). */
  clearIdentity: () => void;
}

export const useIdentityStore = create<IdentityState>()(
  persist(
    (set) => ({
      npub: null,
      accountIndex: 0,
      setIdentity: (npub, accountIndex = 0) => set({ npub, accountIndex }),
      clearIdentity: () => set({ npub: null }),
    }),
    {
      name: "uplink-identity",
      // Only persist the public npub + account index; never secret material
      partialize: (s) => ({ npub: s.npub, accountIndex: s.accountIndex }),
    }
  )
);
