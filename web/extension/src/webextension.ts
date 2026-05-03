type MessageCallback = (response: unknown) => void;
type MessageListener = (
  message: unknown,
  sender: unknown,
  sendResponse: MessageCallback,
) => boolean | void;

type PortListener = (port: ExtensionPort) => void;

export type ExtensionPort = {
  name: string;
  postMessage(message: unknown): void;
  disconnect(): void;
  onMessage: {
    addListener(listener: (message: unknown) => void): void;
  };
};

type StorageArea = {
  get(
    keys?: string | string[] | Record<string, unknown> | null,
  ): Promise<unknown>;
  set(items: Record<string, unknown>): Promise<void>;
  remove(keys: string | string[]): Promise<void>;
};

type CallbackStorageArea = {
  get(
    keys: string | string[] | Record<string, unknown> | null,
    callback: (items: unknown) => void,
  ): void;
  set(items: Record<string, unknown>, callback?: () => void): void;
  remove(keys: string | string[], callback?: () => void): void;
};

type PromiseRuntime = {
  sendMessage(message: unknown): Promise<unknown>;
  connect(connectInfo: { name: string }): ExtensionPort;
  openOptionsPage?: () => Promise<void>;
  onMessage: { addListener(listener: MessageListener): void };
  onConnect: { addListener(listener: PortListener): void };
};

type CallbackRuntime = {
  sendMessage(message: unknown, callback: (response: unknown) => void): void;
  connect(connectInfo: { name: string }): ExtensionPort;
  openOptionsPage?: (callback?: () => void) => void;
  lastError?: { message?: string };
  onMessage: { addListener(listener: MessageListener): void };
  onConnect: { addListener(listener: PortListener): void };
};

type PromiseApi = {
  runtime: PromiseRuntime;
  storage: { local: StorageArea };
};

type CallbackApi = {
  runtime: CallbackRuntime;
  storage: { local: CallbackStorageArea };
};

type GlobalWithExtensionApi = typeof globalThis & {
  browser?: PromiseApi;
  chrome?: CallbackApi;
};

export function getExtensionApi() {
  const global = globalThis as GlobalWithExtensionApi;
  const promiseApi = global.browser;
  const callbackApi = global.chrome;

  if (!promiseApi && !callbackApi) {
    throw new Error("WebExtension API is not available.");
  }

  return {
    runtime: {
      sendMessage(message: unknown) {
        if (promiseApi) {
          return promiseApi.runtime.sendMessage(message);
        }

        return new Promise<unknown>((resolve, reject) => {
          callbackApi?.runtime.sendMessage(message, (response) => {
            const error = callbackApi.runtime.lastError;
            if (error) {
              reject(new Error(error.message || "Extension message failed."));
              return;
            }
            resolve(response);
          });
        });
      },
      connect(connectInfo: { name: string }) {
        return (promiseApi ?? callbackApi)?.runtime.connect(connectInfo);
      },
      openOptionsPage() {
        if (promiseApi?.runtime.openOptionsPage) {
          return promiseApi.runtime.openOptionsPage();
        }

        return new Promise<void>((resolve, reject) => {
          const openOptionsPage = callbackApi?.runtime.openOptionsPage;
          if (!openOptionsPage) {
            reject(new Error("Options page is not supported."));
            return;
          }
          openOptionsPage(() => {
            const error = callbackApi.runtime.lastError;
            if (error) {
              reject(new Error(error.message || "Failed to open options."));
              return;
            }
            resolve();
          });
        });
      },
      onMessage: (promiseApi ?? callbackApi)?.runtime.onMessage,
      onConnect: (promiseApi ?? callbackApi)?.runtime.onConnect,
    },
    storage: {
      local: {
        get(keys?: string | string[] | Record<string, unknown> | null) {
          if (promiseApi) {
            return promiseApi.storage.local.get(keys);
          }

          return new Promise<unknown>((resolve, reject) => {
            callbackApi?.storage.local.get(keys ?? null, (items) => {
              const error = callbackApi.runtime.lastError;
              if (error) {
                reject(new Error(error.message || "Storage read failed."));
                return;
              }
              resolve(items);
            });
          });
        },
        set(items: Record<string, unknown>) {
          if (promiseApi) {
            return promiseApi.storage.local.set(items);
          }

          return new Promise<void>((resolve, reject) => {
            callbackApi?.storage.local.set(items, () => {
              const error = callbackApi.runtime.lastError;
              if (error) {
                reject(new Error(error.message || "Storage write failed."));
                return;
              }
              resolve();
            });
          });
        },
        remove(keys: string | string[]) {
          if (promiseApi) {
            return promiseApi.storage.local.remove(keys);
          }

          return new Promise<void>((resolve, reject) => {
            callbackApi?.storage.local.remove(keys, () => {
              const error = callbackApi.runtime.lastError;
              if (error) {
                reject(new Error(error.message || "Storage remove failed."));
                return;
              }
              resolve();
            });
          });
        },
      },
    },
  };
}
