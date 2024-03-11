import { defineStore } from 'pinia'
import { ref, type Ref } from 'vue'

type Player = { name: string; index: number }

interface BracketCreationForm {
  name: string
  playerList: Player[]
}

export const useBracketStore = defineStore(
  'bracket',
  () => {
    const id: Ref<string | undefined> = ref(undefined)
    const bracket: Ref<Bracket | undefined> = ref(undefined)
    const isSaved: Ref<boolean> = ref(true)
    const formCreate: Ref<BracketCreationForm> = ref({
      playerList: [],
      name: '',
    })
    const counter = ref(0)

    function setBracketId(newId: string) {
      id.value = newId
    }

    function addPlayerInForm(name: string): void {
      counter.value = counter.value + 1
      formCreate.value.playerList.push({ name: name, index: counter.value })
    }

    function removePlayerInForm(index: number): void {
      let player = formCreate.value.playerList.findIndex(
        (p) => p.index === index
      )
      if (player > -1) {
        formCreate.value.playerList.splice(player, 1)
      }
    }

    function removeAllPlayersInForm(): void {
      formCreate.value.playerList = []
    }

    /**
     * when logged in, updates bracket id. Otherwise, gets bracket details
     * @param playerList
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
            bracket_name: formCreate.value.name,
            names: formCreate.value.playerList.map((p) => p.name),
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
          formCreate.value = { playerList: [], name: '' }
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
            let newBracket = await response.json()
            bracket.value = newBracket
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
      bracket,
      isSaved,
      formCreate,
    }
  },
  {
    persist: true,
  }
)
