import { defineStore } from 'pinia'
import { reactive, ref, type Ref } from 'vue'

export type LoginAttempt = 200 | 401 | 404 | 500
interface UserInfos {
  email: string
  name: string
}

export const useUserStore = defineStore(
  'user',
  () => {
    /**
     * ID of user if logged in
     */
    const id: Ref<string | null> = ref(null)
    const infos: UserInfos = reactive({ email: '', name: '' })

    /**
     * @param values
     * @returns 200 when ok, a string if a legible error message is available or 500 when error is not legible
     */
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
          return 500
        }
      } catch (e) {
        console.error(e)
        return 500
      }
    }

    /**
     * Update user ID in store
     * @param values form values
     * @returns status code response
     */
    async function login(values: any): Promise<LoginAttempt> {
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
          return 200
        } else {
          throw new Error('expected json response')
        }
      } else if (response.status === 401) {
        console.warn('bad password')
        return 401
      } else if (response.status === 404) {
        console.warn('unknown email')
        return 404
      } else {
        console.error(`non-200 response for /api/login ${response.status}`)
        return 500
      }
    }

    async function logout() {
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
      } else {
        throw new Error('non-200 response for /api/logout')
      }
    }

    async function deleteAccount(): Promise<boolean> {
      try {
        let response = await fetch(
          `${import.meta.env.VITE_API_URL}/api/users`,
          {
            method: 'DELETE',
          }
        )
        if (response.ok) {
          console.info('successful account deletion')
          id.value = null

          return true
        }
      } catch (e) {
        console.error(e)
        return false
      }
      return false
    }

    async function getUser(): Promise<void> {
      let response = await fetch(`${import.meta.env.VITE_API_URL}/api/users`, {
        method: 'GET',
        headers: {
          Accept: 'application/json',
          'Content-Type': 'application/json',
        },
      })
      if (response.ok) {
        let userInfos: { email: string; name: string } = await response.json()
        console.debug(JSON.stringify(userInfos))
        infos.email = userInfos.email
        infos.name = userInfos.name
      } else {
        throw new Error('non-200 response for /api/users')
      }
    }

    return {
      registration,
      id,
      login,
      logout,
      deleteAccount,
      getUser,
      infos,
    }
  },
  {
    persist: true,
  }
)
