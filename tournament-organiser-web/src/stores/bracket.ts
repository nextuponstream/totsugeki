import { defineStore } from 'pinia'
import { ref, computed, type Ref } from 'vue'

type Player = { name: string; index: number }

export const useBracketStore = defineStore('bracket', () => {
  const id: Ref<string | undefined> = ref(undefined)
  const bracket: Ref<Bracket | undefined> = ref(undefined)

  function setBracketId(newId: string) {
    id.value = newId
  }

  async function createBracket(playerList: Player[]) {
    try {
      let response = await fetch(
        `${import.meta.env.VITE_API_URL}/api/brackets`,
        {
          method: 'POST',
          headers: {
            Accept: 'application/json',
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            names: playerList.map((p) => p.name),
          }),
          // can't send json without cors... https://stackoverflow.com/a/45655314
          // documentation: https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch#supplying_request_options
        }
      )
      if (response.ok) {
        let r = await response.json()
        console.debug(r)
        id.value = r.id
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

  return {
    id,
    setBracketId,
    getBracket,
    createBracket,
    getDisplayableBracket,
    bracket,
  }
})
