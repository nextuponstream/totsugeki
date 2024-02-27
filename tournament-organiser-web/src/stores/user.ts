import { defineStore } from 'pinia'
import { ref, type Ref } from 'vue'

export type LoginAttempt = 200 | 404 | 500

export const useUserStore = defineStore('user', () => {
  const KEY = 'userId'

  /**
   * ID of user if logged in
   */
  const id: Ref<string | null> = ref(null)

  async function registration(values: any): Promise<string | 200 | 500> {
    try {
      let response = await fetch(
        `${import.meta.env.VITE_API_URL}/api/register`,
        {
          method: 'POST',
          headers: {
            Accept: 'application/json',
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ ...values }),
        }
      )
      if (response.ok) {
        console.info('successful registration')
        return 200
      } else if (response.status === 400) {
        let errorMessage: { message: string } = await response.json()
        return errorMessage.message
      } else {
        throw new Error('non-200 response for /api/report-result-for-bracket')
      }
    } catch (e) {
      console.error(e)
      return 500
    }
  }

  /**
   * Rely on localstorage to persist some state in between page reload
   */
  function setUserId() {
    id.value = localStorage.getItem(KEY)
  }

  /**
   * Update user ID in store
   * @param values form values
   * @returns status code response
   */
  async function login(values: any): Promise<LoginAttempt> {
    try {
      let response = await fetch(`${import.meta.env.VITE_API_URL}/api/login`, {
        method: 'POST',
        headers: {
          Accept: 'application/json',
          'Content-Type': 'application/json',
          Authorization: 'Basic ' + btoa(`${values.email}:${values.password}`),
        },
        credentials: 'same-origin',
        body: JSON.stringify({ ...values }),
      })
      if (response.ok) {
        console.info('successful login')
        if (response.body) {
          let body = await response.json()
          // store user ID and prefer logged in view from now on
          id.value = body.user_id
          localStorage.setItem(KEY, body.user_id)
          return 200
        } else {
          throw new Error('expected json response')
        }
      } else if (response.status === 404) {
        console.warn('unknown email')
        return 404
      } else {
        console.error(`non-200 response for /api/login ${response.status}`)
        return 500
      }
    } catch (e) {
      console.error('could not login')
      throw e
    }
  }

  async function logout() {
    try {
      let response = await fetch(`${import.meta.env.VITE_API_URL}/api/logout`, {
        method: 'POST',
        headers: {
          Accept: 'application/json',
          'Content-Type': 'application/json',
        },
        credentials: 'same-origin',
      })
      if (response.ok) {
        console.info('successful logout')
        id.value = null
        localStorage.removeItem(KEY)
      } else {
        throw new Error('non-200 response for /api/logout')
      }
    } catch (e) {
      console.error(e)
    }
  }

  async function deleteAccount(): Promise<boolean> {
    try {
      let response = await fetch(`${import.meta.env.VITE_API_URL}/api/users`, {
        method: 'DELETE',
      })
      if (response.ok) {
        console.info('successful account deletion')
        localStorage.removeItem(KEY)
        id.value = null

        return true
      }
    } catch (e) {
      console.error(e)
      return false
    }
    return false
  }

  return {
    registration,
    id,
    setUserId,
    login,
    logout,
    deleteAccount,
  }
})
