// TODO all http request should log out the error on 400-500

import { useUserStore } from '@/stores/user'
import router, { RouteNames } from '@/router'

// const userStore = useUserStore()

type ALLOWED_METHOD = 'GET' | 'POST' | 'PUT' | 'DELETE'

class HttpClient {
  baseApiUrl: string = import.meta.env.VITE_API_URL
  apiHeaders = {
    Accept: 'application/json',
    'Content-Type': 'application/json',
  }

  /**
   * @param path
   * @returns json response
   * @throws Error when response status code is not 200
   */
  async get(path: string): Promise<Response> {
    return await this.fetchResponse('GET', path)
  }

  async post(path: string, data?: any): Promise<any> {
    return await this.fetchResponse('POST', path, data)
  }

  async put(path: string) {
    return await this.fetchResponse('PUT', path)
  }

  /**
   *
   * @param path
   */
  async delete(path: string): Promise<any> {
    return await this.fetchResponse('DELETE', path)
  }

  private async fetchResponse(
    method: ALLOWED_METHOD,
    path: string,
    data?: any
  ): Promise<any> {
    let response = await fetch(`${this.baseApiUrl}${path}`, {
      method,
      headers: this.apiHeaders,
      body: data,
    })
    if (!response.ok) {
      await this.handleErrors(response.status, path, response)
    }
    return response
  }

  /**
   * Automatically redirect user to home page after receiving 401
   * @param status
   * @param path
   * @param response
   * @throws Error when error is unrecoverable and caller should be interrupted
   */
  private async handleErrors(status: number, path: string, response: Response) {
    console.error(`response status code is not ok: ${status} for path ${path}`)
    console.debug(await response.text())
    if (status === 401) {
      // nice workaround to avoid calling the userStore when store may not be
      // initialised
      await router.push({ name: RouteNames.logout })
    } else {
      // handle error by not handling it and interrupt everything rather than
      // keep going
      throw new Error(`received status ${status} for path ${path}`)
    }
  }
}

export const httpClient = new HttpClient()
