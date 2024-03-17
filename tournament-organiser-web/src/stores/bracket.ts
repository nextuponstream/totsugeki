import { defineStore } from 'pinia'
import { ref, type Ref } from 'vue'

type Player = { name: string; index: number }

interface BracketCreationForm {
  bracket_name: string
  player_names: Player[]
}

interface MatchResult {
  players: { name: string; id: string }[]
  scoreP1: number
  scoreP2: number
}

export const useBracketStore = defineStore(
  'bracket',
  () => {
    const id: Ref<string | undefined> = ref(undefined)
    const bracket: Ref<Bracket | undefined> = ref(undefined)
    const isSaved: Ref<boolean> = ref(true)
    const formCreate: Ref<BracketCreationForm> = ref({
      player_names: [],
      bracket_name: '',
    })
    const counter = ref(0)
    const reportedResults: Ref<MatchResult[]> = ref([])

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
     * when logged in, updates bracket id. Otherwise, gets bracket details
     * @param loggedIn
     */
    async function createBracket(loggedIn: boolean) {
      console.debug(`creating bracket with ${loggedIn ? 'user' : 'guest'}`)
      try {
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
          // can't send json without cors... https://stackoverflow.com/a/45655314
          // documentation: https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch#supplying_request_options
        })
        if (response.ok) {
          let r = await response.json()
          console.debug(r)
          if (loggedIn) {
            id.value = r.id
            bracket.value = undefined
            isSaved.value = true
          } else {
            id.value = undefined
            bracket.value = r
            isSaved.value = false
          }
          formCreate.value = { player_names: [], bracket_name: '' }
        } else {
          throw new Error(
            `response (${
              response.status
            }) \"${await response.text()}\" from POST /api/brackets`
          )
        }
      } catch (e) {
        console.error(e)
      }
    }

    async function getBracket() {
      try {
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
      } catch (e) {
        console.error(e)
      }
    }

    async function getDisplayableBracket() {
      try {
        let response = await fetch(
          `${import.meta.env.VITE_API_URL}/api/brackets/${id.value}/display`,
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
            }) \"${await response.text()}\" from /api/brackets/${
              id.value
            }/display`
          )
        }
      } catch (e) {
        console.error(e)
      }
    }

    async function reportResult(
      players: { name: string; id: string }[],
      scoreP1: number,
      scoreP2: number
    ) {
      if (bracket.value) {
        try {
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
          } else {
            console.debug(await response.text())
            throw new Error(
              'non-200 response for /api/report-result-for-bracket'
            )
          }
        } catch (e) {
          console.error(e)
        }
      } else {
        throw new Error('missing bracket in store for reporting result')
      }
    }

    /**
     * Someone did a bracket: "Oh no, I kinda want to save that actually"
     * Then store all the steps done and replay them server-side to ensure it's
     * actually a valid bracket.
     */
    async function saveBracket() {
      // use /brackets/save endpoint
      if (reportedResults.value && bracket.value?.bracket?.participants) {
        try {
          console.debug(`submitting result for bracket...`)
          let player_names = bracket.value.bracket.participants.map(
            (p) => p.name
          )
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
                player_names: player_names,
              }),
            }
          )
          if (response.ok) {
            reportedResults.value = []
          } else {
            console.debug(await response.text())
            throw new Error('non-200 response for /api/brackets/save')
          }
        } catch (e) {
          console.error(e)
        }
      } else {
        throw new Error('missing bracket in store for reporting result')
      }
    }

    return {
      id,
      setBracketId,
      getBracket,
      createBracket,
      getDisplayableBracket,
      reportResult,
      addPlayerInForm,
      removePlayerInForm,
      removeAllPlayersInForm,
      saveBracket,
      bracket,
      isSaved,
      formCreate,
    }
  },
  {
    persist: true,
  }
)
