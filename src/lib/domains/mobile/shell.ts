/** True only in the Portal Mobile build (`VITE_PORTAL_MOBILE=1`); desktop is unaffected. */
export const isMobileShell = import.meta.env.VITE_PORTAL_MOBILE === '1';
