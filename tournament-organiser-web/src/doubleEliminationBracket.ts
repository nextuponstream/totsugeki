import type { Match } from '@/match'

export interface Lines {
  left_border: boolean
  bottom_border: boolean
}

interface Player {
  id: string
  name: string
}

export interface RawBracket {
  id?: string
  name?: string

  // ordered Player IDs
  seeding?: string[]
}

export type Participants = Player[]

export interface DoubleEliminationBracket {
  winner_bracket: Match[][]
  winner_bracket_lines: Lines[][]
  loser_bracket: Match[][]
  loser_bracket_lines: Lines[][]
  grand_finals: Match | undefined
  grand_finals_reset: Match | undefined
  bracket: RawBracket | undefined
  tournament: Tournament | undefined
  is_participant: boolean
  is_tournament_organiser: boolean
  tournament_id: string
  tournament_name: string
}

interface Tournament {
  id: string
  name: string
  participants: Participants
}
