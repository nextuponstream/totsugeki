export interface Match {
  id: string
  players: { name: string; id: string }[]
  seeds: number[]
  score?: number[]
  row_hint: number | null
}
