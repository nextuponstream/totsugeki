export const BASE_URL = import.meta.env.VITE_BASE_URL ?? "http://localhost:5173"
export const AXUM_ENV = import.meta.env.VITE_AXUM_ENV ?? "development"

export const config: {
    axumHeaders: RequestInit,
} = {
    axumHeaders: AXUM_ENV === 'development' ? {mode: 'no-cors'} : {mode: undefined}
}

export default config
