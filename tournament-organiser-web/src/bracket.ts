interface Match {
    id: string,
    players: string[],
    seeds: number[],
    score: number[],
    row_hint: number | null,
  }

  interface Bracket {
    winner_bracket: Match[][],
    loser_bracket: Match[][],
    grand_finals: Match | null,
    grand_finals_reset: Match | null,
  }