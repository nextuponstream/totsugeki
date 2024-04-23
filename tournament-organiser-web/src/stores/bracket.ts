import { defineStore } from 'pinia'
import { ref, type Ref } from 'vue'

type Player = { name: string; index: number }

interface BracketCreationForm {
  bracket_name: string
  player_names: Player[]
}

interface MatchResult {
  p1_id: string
  p2_id: string
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

export const useBracketStore = defineStore(
  'bracket',
  () => {
    const id: Ref<string | undefined> = ref(undefined)
    const bracket: Ref<Bracket | undefined> = ref(undefined)
    const bracketList: Ref<Bracket[] | undefined> = ref(undefined)
    const isSaved: Ref<boolean> = ref(true)
    const formCreate: Ref<BracketCreationForm> = ref({
      player_names: [],
      bracket_name: '',
    })
    const counter = ref(0)
    const reportedResults: Ref<MatchResult[]> = ref([])
    const pagination: Ref<Pagination> = ref({
      limit: 10,
      offset: 0,
      sortOrder: 'DESC',
      total: 0,
    })

    function setBracketId(newId: string) {
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
    async function createBracket(loggedIn: boolean) {
      console.debug(`creating bracket with ${loggedIn ? 'user' : 'guest'}`)
      let url = `${import.meta.env.VITE_API_URL}/api/${
        loggedIn ? '' : 'guest/'
      }brackets`
      let response = await fetch(url, {
        method: 'POST',
        headers: {
          Accept: 'application/json',
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          bracket_name: formCreate.value.bracket_name,
          player_names: formCreate.value.player_names.map((p) => p.name),
        }),
      })
      if (response.ok) {
        let r = await response.json()
        console.debug(r)
        if (loggedIn) {
          id.value = r.id
          isSaved.value = true
        } else {
          id.value = undefined
          bracket.value = r
          isSaved.value = false
        }
        reportedResults.value = []
        formCreate.value = { player_names: [], bracket_name: '' }
      } else {
        throw new Error(
          `response (${
            response.status
          }) \"${await response.text()}\" from POST /api/brackets`
        )
      }
    }

    /**
     * Fetch bracket details depending on bracket ID in store.
     * @throws Error when something goes wrong with the API
     */
    async function getDisplayableBracket() {
      let response = await fetch(
        `${import.meta.env.VITE_API_URL}/api/brackets/${id.value}`,
        {
          method: 'GET',
          headers: {
            Accept: 'application/json',
            'Content-Type': 'application/json',
          },
        }
      )
      if (response.ok) {
        let r = await response.json()
        console.debug(r)
        bracket.value = r
      } else {
        throw new Error(
          `response (${
            response.status
          }) \"${await response.text()}\" from /api/brackets/${id.value}`
        )
      }
    }

    /**
     * Given a match with two `players`, report result
     * @param players
     * @param scoreP1
     * @param scoreP2
     * @throws Error when something goes wrong with the API
     */
    async function reportResult(
      players: { name: string; id: string }[],
      scoreP1: number,
      scoreP2: number
    ) {
      if (bracket.value) {
        console.debug(`submitting result for bracket...`)
        let response = await fetch(
          `${import.meta.env.VITE_API_URL}/api/report-result-for-bracket`,
          {
            method: 'POST',
            headers: {
              Accept: 'application/json',
              'Content-Type': 'application/json',
            },
            body: JSON.stringify({
              bracket: bracket.value.bracket,
              p1_id: players[0].id,
              p2_id: players[1].id,
              score_p1: scoreP1,
              score_p2: scoreP2,
            }),
          }
        )
        if (response.ok) {
          bracket.value = await response.json()
          reportedResults.value.push({
            p1_id: players[0].id,
            p2_id: players[1].id,
            score_p1: scoreP1,
            score_p2: scoreP2,
          })
        } else {
          console.debug(await response.text())
          throw new Error('non-200 response for /api/report-result-for-bracket')
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
    async function saveBracket() {
      // use /brackets/save endpoint
      if (reportedResults.value && bracket.value?.bracket?.participants) {
        console.debug(`submitting result for bracket...`)
        let player_names = bracket.value.bracket.participants
        let response = await fetch(
          `${import.meta.env.VITE_API_URL}/api/brackets/save`,
          {
            method: 'POST',
            headers: {
              Accept: 'application/json',
              'Content-Type': 'application/json',
            },
            body: JSON.stringify({
              bracket_name: bracket.value?.bracket?.name,
              results: reportedResults.value,
              players: player_names,
            }),
          }
        )
        if (response.ok) {
          reportedResults.value = []
          isSaved.value = true
          bracket.value = await response.json()
        } else {
          console.debug(await response.text())
          throw new Error('non-200 response for /api/brackets/save')
        }
      } else {
        throw new Error('missing bracket in store for reporting result')
      }
    }

    /**
     * Fetches bracket of user using current pagination state
     * @param userId
     */
    async function getBracketsFrom(userId: string) {
      let response = await fetch(
        `${import.meta.env.VITE_API_URL}/api/user/${userId}/brackets?limit=${
          pagination.value.limit
        }&offset=${pagination.value.offset}&sort_order=${
          pagination.value.sortOrder
        }`,
        {
          method: 'GET',
          headers: {
            Accept: 'application/json',
            'Content-Type': 'application/json',
          },
        }
      )
      if (response.ok) {
        let paginationResult: PaginationResponse = await response.json()
        bracketList.value = paginationResult.data
        pagination.value.total = paginationResult.total
      } else {
        console.debug(await response.text())
        throw new Error(`non-200 response for /api/user/${userId}/brackets`)
      }
    }

    return {
      id,
      setBracketId,
      createBracket,
      getDisplayableBracket,
      reportResult,
      addPlayerInForm,
      removePlayerInForm,
      removeAllPlayersInForm,
      saveBracket,
      getBracketsFrom,
      bracket,
      bracketList,
      isSaved,
      formCreate,
      reportedResults, // export ref so localStorage is updated with that value
      pagination,
    }
  },
  {
    persist: true,
  }
)
