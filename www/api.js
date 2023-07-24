// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

// @ts-check
/**
 * @typedef {number|number[]} ReqAddr
 * @typedef {'GMSJ01'|'GMSE01'|'GMSP01'|'GMSJ0A'} SMSVersion
 */

/** @param {string} s */
const hex2dv = s => new DataView(Uint8Array.from(
  (s.match(/../g) ?? /**@type{string[]}*/([]))
    .map(s => parseInt(s, 16))).buffer
);

/**
 * @param {{
 *   onClose?: null | ((this: WebSocket, ev: CloseEvent)=>void)
 * }} options
 */
function Client({onClose = null}={}) {
  /** @type {Map<number, {rsv: (res: any)=>void, rjt: (res: any)=>void}>} */
  const reqs = new Map();
  /** @type {WebSocket|null} */
  let ws = null;
  let nextId = 1;

  /**
   * @template T
   * @param {string} action
   * @param {any} [payload]
   * @returns {Promise<T>}
   */
  const request = (action, payload=null) => new Promise((rsv, rjt) => {
    if (ws == null) throw Error('Client is not connected to server. Use `client.connect()` first.');
    const id = nextId++;
    reqs.set(id, {rsv, rjt});
    ws.send(JSON.stringify([id, action, payload]));
  });

  return {
    connect: (url=`ws://${window.location.host}/`, protocol=undefined) => new Promise((rsv, rjt) => {
      const ws1 = new WebSocket(url, protocol);
      ws1.onmessage = ({data}) => {
        if (typeof data !== 'string') return; // TODO
        const [id, body] = JSON.parse(data);
        if (id > 0) {
          reqs.get(id)?.rsv(body);
          reqs.delete(id);
        } else {
          reqs.get(-id)?.rjt(body);
          reqs.delete(-id);
        }
      };
      ws1.onopen = rsv;
      ws1.onerror = rjt; // TODO auto reconnect
      ws1.onclose = onClose;
      ws = ws1;
    }),
    get ws() {return ws},
    request,
    api: {
      /**
       * @returns {Promise<number|null>}
       */
      init: () => request('init'),

      /**
       * @param {ReqAddr} addr
       * @param {string} type
       */
      read: (addr, type) => request('read', {
        addr: addr instanceof Array ? addr : [addr],
        type,
      }).then((/**@type{string[]|string|null}*/s) => s),

      /**
       * @param {ReqAddr} addr
       * @param {number} size
       */
      readBytes: (addr, size) => request('read', {
        addr: addr instanceof Array ? addr : [addr],
        size,
      }).then((/**@type{string|null}*/s) => s == null ? null : hex2dv(s)),

      /**
       * @param {ReqAddr} addr
       * @returns {Promise<string|null>}
       */
      readString: addr => request('readString', {
        addr: typeof addr === 'number' ? [addr] : addr,
      }),

      /**
       * @param {ReqAddr} addr
       * @param {string|ArrayBuffer|ArrayBufferView} payload
       * @returns {Promise<boolean>}
       */
      write: (addr, payload) => request('write', {
        addr: typeof addr === 'number' ? [addr] : addr,
        payload: typeof payload === 'string' ? payload : Array.from(
          new Uint8Array(payload instanceof ArrayBuffer ? payload : payload.buffer),
          x => x.toString(16).padStart(2, '0'),
        ).join(''),
      }),

      /**
       * @param {ReqAddr} addr
       * @returns {Promise<string|null>}
       */
      getClass: addr => request('getClass', {
        addr: typeof addr === 'number' ? [addr] : addr,
      }),

      /**
       * @param {string} type
       * @returns {Promise<[
       *   offsets: string,
       *   name: string,
       *   notes: string,
       *   type: string,
       *   class_: string,
       * ][]>}
       */
      getFields: type => request('getFields', type),

      getManagers: () => request('getManagers')
        .then((/**@type{[addr: number, type: string, name: string, count: number][]|null}*/rows) =>
          rows?.map(row => ({addr: row[0], type: row[1], name: row[2], count: row[3]})) ?? []),

      /**
       * @param {ReqAddr} addr
       */
      getManagees: addr => request('getManagees', addr)
        .then((/**@type{[addr: number, type: string, name: string][]|null}*/rows) =>
          rows?.map(row => ({addr: row[0], type: row[1], name: row[2]})) ?? []),

      /** @returns {Promise<SMSVersion>} */
      getVersion: ()  => request('getVersion'),

      reload: () => request('reload', null),
    },
  };
}
