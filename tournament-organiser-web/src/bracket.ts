interface Match {
  id: string
  players: { name: string; id: string }[]
  seeds: number[]
  score: number[]
  row_hint: number | null
}

interface Lines {
  left_border: boolean
  bottom_border: boolean
}

interface Bracket {
  winner_bracket: Match[][]
  winner_bracket_lines: Lines[][]
  loser_bracket: Match[][]
  loser_bracket_lines: Lines[][]
  grand_finals: Match | undefined
  grand_finals_reset: Match | undefined
  bracket: Object | undefined
}
