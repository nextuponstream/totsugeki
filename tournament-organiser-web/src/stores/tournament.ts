import { defineStore } from 'pinia'
import { ref, type Ref } from 'vue'
import { httpClient } from '@/httpClient'
import type {
  DoubleEliminationBracket,
  Participants,
  RawBracket,
} from '@/doubleEliminationBracket'

type Player = { name: string; index: number }

interface TournamentCreationForm {
  tournament_name: string
  player_names: Player[]
}

interface MatchResult {
  player1_id: string
  player2_id: string
  score_p1: number
  score_p2: number
}

type PaginationLimit = 10 | 25 | 50 | 100
type SortOrder = 'ASC' | 'DESC'

interface Pagination {
  limit: PaginationLimit
  offset: number
  sortOrder: SortOrder
  total: number
}

interface PaginationResponse {
  total: number
  data: any
}

export const useTournamentStore = defineStore(
  'tournament',
  () => {
    const id: Ref<string | undefined> = ref(undefined)
    const bracket: Ref<DoubleEliminationBracket | undefined> = ref(undefined)
    const participants: Ref<Participants | undefined> = ref(undefined)
    const tournamentList: Ref<DoubleEliminationBracket[] | undefined> =
      ref(undefined)
    const isSaved: Ref<boolean> = ref(true)
    const formCreate: Ref<TournamentCreationForm> = ref({
      player_names: [],
      tournament_name: '',
    })
    const counter = ref(0)
    const reportedResults: Ref<MatchResult[]> = ref([])
    const pagination: Ref<Pagination> = ref({
      limit: 10,
      offset: 0,
      sortOrder: 'DESC',
      total: 0,
    })

    function setTournamentId(newId: string) {
      id.value = newId
    }

    function addPlayerInForm(name: string): void {
      counter.value = counter.value + 1
      formCreate.value.player_names.push({ name: name, index: counter.value })
    }

    function removePlayerInForm(index: number): void {
      let player = formCreate.value.player_names.findIndex(
        (p) => p.index === index
      )
      if (player > -1) {
        formCreate.value.player_names.splice(player, 1)
      }
    }

    function removeAllPlayersInForm(): void {
      formCreate.value.player_names = []
    }

    /**
     * When logged in, updates bracket id, then you can visit corresponding page. Otherwise, put bracket in store and
     * visit guest page to display those details
     * @param loggedIn
     * @throws Error when something goes wrong with the API
     */
    async function createTournament(loggedIn: boolean) {
      console.debug(`creating tournament with ${loggedIn ? 'user' : 'guest'}`)
      let url = `/${loggedIn ? '' : 'guests/'}tournaments`
      let response = await httpClient.post(url, {
        tournament_name: formCreate.value.tournament_name,
        player_names: formCreate.value.player_names.map((p) => p.name),
      })
      let r = await response.json()
      console.debug(r)
      if (loggedIn) {
        id.value = r.id
        isSaved.value = true
      } else {
        id.value = undefined
        bracket.value = r
        participants.value = r.participants
        isSaved.value = false
      }
      reportedResults.value = []
      formCreate.value = { player_names: [], tournament_name: '' }
    }

    /**
     * Fetch bracket details depending on bracket ID in store.
     * @throws Error when something goes wrong with the API
     */
    async function getDisplayableTournament() {
      let response = await httpClient.get(`/tournaments/${id.value}`)
      let r = await response.json()
      console.debug('updating tournament store', r)
      // console.log(bracket.value?.winner_bracket)
      // console.log(r.winner_bracket)
      bracket.value = r
      // FIXME
      bracket.value!.is_tournament_organiser = r.is_tournament_organiser
      bracket.value!.tournament_name = r.tournament.name
      bracket.value!.tournament = r.tournament
      bracket.value!.tournament_id = r.tournament.id

      console.log(bracket.value?.bracket?.seeding)
      participants.value = r.participants
    }

    /**
     * Given a match with two `players`, report result
     * @param players
     * @param scoreP1
     * @param scoreP2
     * @param dryRun true if you must not be saved to database
     * @throws Error when something goes wrong with the API
     */
    async function reportResult(
      players: { name: string; id: string }[],
      scoreP1: number,
      scoreP2: number,
      dryRun: boolean
    ) {
      // FIXME after reporting result while logged in on a bracket that belongs
      //  to the user, hit f5 and all results should still be there
      if (bracket.value) {
        console.debug(`submitting result for bracket...`)
        let path = dryRun ? `/report-result` : `/tournaments/${id.value}/score`

        let response = await httpClient.post(path, {
          bracket: bracket.value.bracket,
          tournament: bracket.value.tournament,
          player1_id: players[0].id,
          player2_id: players[1].id,
          score_p1: scoreP1,
          score_p2: scoreP2,
        })
        bracket.value = await response.json()
        reportedResults.value.push({
          player1_id: players[0].id,
          player2_id: players[1].id,
          score_p1: scoreP1,
          score_p2: scoreP2,
        })

        let r = response
        if (!dryRun) {
          // FIXME
          bracket.value!.tournament_id = bracket.value!.tournament!.id
        }
      } else {
        throw new Error('missing bracket in store for reporting result')
      }
    }

    /**
     * Someone did a bracket: "Oh no, I kinda want to save that actually"
     * Then store all the steps done and replay them server-side to ensure it's
     * actually a valid bracket.
     * @throws Error when something goes wrong with the API
     */
    async function saveTournament() {
      // use /tournaments/save endpoint
      if (reportedResults.value && bracket.value?.bracket?.seeding) {
        console.debug(`submitting result for bracket...`)
        let response = await httpClient.post(`/tournaments/save`, {
          // FIXME tournament_name
          bracket_name: bracket.value?.tournament?.name,
          results: reportedResults.value,
          // FIXME use players from tournament and delete participants
          players: participants.value,
        })
        reportedResults.value = []
        isSaved.value = true
        bracket.value = await response.json()
        // save tournament aside from bracket
      } else {
        throw new Error('missing bracket in store for reporting result')
      }
    }

    /**
     * Fetches bracket of user using current pagination state
     * @param userId
     */
    async function getBracketsFrom(userId: string) {
      let response = await httpClient.get(
        `/user/${userId}/tournaments?limit=${pagination.value.limit}&offset=${pagination.value.offset}&sort_order=${pagination.value.sortOrder}`
      )
      let paginationResult: PaginationResponse = await response.json()
      tournamentList.value = paginationResult.data
      pagination.value.total = paginationResult.total
    }

    async function join() {
      let response = await httpClient.post(`/tournaments/${id.value}/join`, {})
      throw new Error('implement')
    }

    return {
      id,
      setTournamentId,
      createTournament,
      getDisplayableTournament,
      reportResult,
      addPlayerInForm,
      removePlayerInForm,
      removeAllPlayersInForm,
      saveTournament,
      getBracketsFrom,
      bracket,
      bracketList: tournamentList,
      isSaved,
      formCreate,
      reportedResults, // export ref so localStorage is updated with that value
      pagination,
      participants,
      join,
    }
  },
  {
    persist: true,
  }
)
