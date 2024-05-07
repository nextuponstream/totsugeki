import { defineStore } from 'pinia'
import { reactive, ref, type Ref } from 'vue'
import { httpClient } from '@/httpClient'

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
        let response = await httpClient.post(`/register`, values)
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
     *
     * NOTE: this is too specialised and not reused enough to extract any logic
     * in httpClient because
     * @param values form values
     * @returns status code response
     */
    async function login(values: any): Promise<LoginAttempt> {
      let response = await fetch(`${import.meta.env.VITE_API_URL}/login`, {
        method: 'POST',
        headers: {
          Accept: 'application/json',
          'Content-Type': 'application/json',
          Authorization: 'Basic ' + btoa(`${values.email}:${values.password}`),
        },
        credentials: 'same-origin', // keep any session cookie for future requests
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
      await httpClient.post(`/logout`)
      id.value = null
    }

    async function deleteAccount(): Promise<any> {
      await httpClient.delete('/users')
      console.info('successful account deletion')
      id.value = null
    }

    async function getUser(): Promise<void> {
      let response = await httpClient.get('/users/profile')
      let userInfos: { email: string; name: string } = await response.json()
      console.debug(JSON.stringify(userInfos))
      infos.email = userInfos.email
      infos.name = userInfos.name
    }

    function loggedIn() {
      return id.value !== null
    }

    return {
      registration,
      id,
      login,
      logout,
      deleteAccount,
      getUser,
      infos,
      loggedIn,
    }
  },
  {
    persist: true,
  }
)
