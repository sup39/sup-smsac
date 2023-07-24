// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

// @ts-check
/**
 * @typedef {{addr: number, type: string, name: string, count: number}} Manager
 * @typedef {{addr: number, type: string, name: string}} Managee
 * @typedef {(td: HTMLTableCellElement) => void} CellFactory
 * @typedef {CellFactory[]} RowFactory
 * @typedef {{name: string, notes: string, offset: string|string[], type: string}} Field
 * @typedef {(Omit<Field, 'offset'> & {offset: number[], srcType: string})} FieldView
 * @typedef {{offsets: Field[]}} ObjParamsDBEntry
 * @typedef {Record<string, null|ObjParamsDBEntry>} ObjParamsDB
 */

const fmt = {
  /** @param {number} x */
  hex: x => x.toString(16).toUpperCase(),
};

/**
 * @param {HTMLTableElement} table
 * @param {((td: HTMLTableCellElement)=>void)[][]} gTable
 */
function initTable(table, gTable) {
  const nRow0 = table.rows.length;
  for (let r=0; r<nRow0; r++) table.deleteRow(-1);
  gTable.forEach(gRow => {
    const row = table.insertRow();
    gRow.forEach(g => g(row.insertCell()));
  });
}

document.addEventListener('DOMContentLoaded', async () => {
  const elmMsg = /**@type {HTMLDivElement}*/(document.getElementById('msg'));
  const btnReloadObjParams = /**@type {HTMLButtonElement}*/(document.getElementById('btnReloadObjParams'));
  const btnReloadManagers = /**@type {HTMLButtonElement}*/(document.getElementById('btnReloadManagers'));
  const cbShowManagers = /**@type {HTMLInputElement}*/(document.getElementById('cbShowManagers'));
  const cbShowObjParamsNotes = /**@type {HTMLInputElement}*/(document.getElementById('cbShowObjParamsNotes'));
  const cbWrapFlex = /**@type {HTMLInputElement}*/(document.getElementById('cbWrapFlex'));

  /** @param {string} msg */
  function showError(msg) {
    elmMsg.textContent = msg;
    document.body.classList.add('error');
  }
  const client = Client({
    onClose: () => showError(`Disconnected from server. Please reload the page.`),
  });
  const {api} = client;
  Object.assign(window, {client, api}); // TODO

  /**************** UI definition ****************/
  /**
   * @param {HTMLElement|null} elm
   */
  function ManagerList(elm) {
    if (elm == null) throw new Error('ManagerList not found');
    const elmTable = (() => {
      const e = elm.querySelector('table');
      if (e == null) throw new Error('table should present in ManagerList');
      return e;
    })();
    // TODO put in a json file
    const staticVariables = [
      {
        name: 'gpApplication',
        addrs: [0x803E6000, 0x803E9700, 0x803E10C0, 0x803DA8E0],
        type: 'TApplication',
      },
      {
        name: 'gpMarDirector',
        addrs: [0x8040A2A8, 0x8040E178, 0x80405840, 0x803FF018],
        type: 'TMarDirector*',
      },
      {
        name: 'QF',
        addrs: [0x8040A2A8, 0x8040E178, 0x80405840, 0x803FF018],
        type: 'TMarDirector@QF*',
      },
      {
        name: 'マリオ',
        addrs: [0x8040A378, 0x8040E0E8, 0x804057B0, 0x803FEF88],
        type: 'TMario*',
      },
    ];
    return {
      get classList() {
        return elm.classList;
      },
      async reload() {
        const [vars, managers] = await Promise.all([
          api.getVersion().then(async ver => {
            const iver = ['GMSJ01', 'GMSE01', 'GMSP01', 'GMSJ0A'].indexOf(ver);
            return await Promise.all(staticVariables.map(async o => {
              const ptrlv = o.type.match(/\*+$/)?.[0].length ?? 0;
              const type = o.type.substring(0, o.type.length-ptrlv);
              const addr0 = o.addrs[iver];
              const addr = ptrlv === 0 ? addr0 : await api.readBytes(
                [addr0].concat(...Array(ptrlv-1).fill(0)), 4,
              ).then(dv => dv?.getUint32(0) ?? 0);
              return {name: o.name, addr, type};
            }));
          }),
          api.getManagers(),
        ]);
        fieldsViewer.reset();
        initTable(elmTable, [
          ...vars.map(o => makeManageesRowFactory(o)),
          ...managers.map(makeManagersRowFactory),
        ]);
      },
    };
  }
  const managerList = ManagerList(document.getElementById('managerList'));

  /**
   * @param {HTMLElement|null} elm
   */
  function FieldsViewer(elm) {
    if (elm == null) throw new Error('FieldsViewer not found');
    const elmTitle = (() => {
      const e = elm.querySelector('h3');
      if (e == null) throw new Error('h3 should present in FieldsViewer');
      return e;
    })();
    const elmTable = (() => {
      const e = elm.querySelector('table');
      if (e == null) throw new Error('table should present in FieldsViewer');
      return e;
    })();
    // states
    const tdidxVal = 2;
    let hAnm = NaN;
    let t0 = 0;
    /** @type {Managee|null} */
    let target = null;
    async function readValues() {
      if (target == null) return [];
      const values = await api.read([target.addr], target.type);
      return values instanceof Array ? values : [values];
    }
    /** @param {DOMHighResTimeStamp} t */
    async function render(t) {
      if (t-t0 >= 33) { // TODO configurable fps
        (await readValues())
          .forEach((s, i) => elmTable.rows[i].cells[tdidxVal].textContent = s);
        t0 = t;
      }
      hAnm = requestAnimationFrame(render);
    }
    const methods = {
      get classList() {
        return elm.classList;
      },
      reload() {
        api.reload().then(() => {
          target != null && methods.view(target);
        }, err => {
          elmMsg.textContent = err;
        });
      },
      reset() {
        elm.classList.add('hidden');
      },
      /** @param {Manager|Managee} o */
      async view(o) {
        cancelAnimationFrame(hAnm);
        target = o;
        elmTitle.textContent = `${o.name} (${o.type}) [${fmt.hex(o.addr)}]`;
        const fields = await api.getFields(o.type);
        const values = readValues();
        initTable(elmTable, fields.map((r, i) => [
          td => td.textContent = r[0],
          td => td.textContent = r[1],
          td => td.textContent = values[i],
          td => td.textContent = r[2],
          td => td.textContent = r[3],
          td => td.textContent = r[4],
        ]));
        elm.classList.remove('hidden');
        if (fields.length) hAnm = requestAnimationFrame(render);
      },
    }
    return methods;
  }
  const fieldsViewer = FieldsViewer(document.getElementById('fieldsViewer'));
  btnReloadObjParams.addEventListener('click', () => {
    fieldsViewer.reload();
  });
  btnReloadManagers.addEventListener('click', async () => {
    managerList.reload();
  });
  cbShowManagers.addEventListener('change', function () {
    managerList.classList[this.checked ? 'remove' : 'add']('hidden');
  });
  cbShowObjParamsNotes.addEventListener('change', function () {
    fieldsViewer.classList[this.checked ? 'add' : 'remove']('showNotes');
  });
  cbWrapFlex.addEventListener('change', function () {
    // TODO
    document.body.classList[this.checked ? 'add' : 'remove']('wrap');
  });

  /** @type {(o: Manager) => RowFactory} */
  const makeManagersRowFactory = o => [
    td => {
      // remove old button
      const btn0 = td.querySelector('button');
      if (btn0) td.removeChild(btn0);
      // create new button
      const btn = document.createElement('button');
      let open = false;
      /** @type {HTMLTableRowElement[] | null} */
      let trManagees = null;
      btn.textContent = '>';
      btn.addEventListener('click', async () => {
        open = !open;
        btn.textContent = open ? 'v' : '>';
        if (open) {
          if (trManagees == null) {
            const managees = await api.getManagees(o.addr);
            const trP = td.parentElement?.parentElement;
            const tr1 = td.parentElement?.nextSibling ?? null;
            trManagees = managees.map((o, i) => {
              const tr = document.createElement('tr');
              tr.classList.add('managee');
              trP?.insertBefore(tr, tr1);
              makeManageesRowFactory(o, i).forEach(f => f(tr.insertCell()));
              return tr;
            });
          } else {
            trManagees.forEach(e => e.classList.remove('hidden'));
          }
        } else {
          trManagees?.forEach(e => e.classList.add('hidden'));
        }
      });
      td.appendChild(btn);
    },
    td => td.textContent = `${o.name} (${o.count})`,
    td => td.textContent = `${o.type}`,
    td => td.textContent = `${fmt.hex(o.addr)}`,
    makeViewerNavigatorFactory(o),
  ];

  /** @type {(o: Managee, i?: number) => ((td: HTMLTableCellElement) => void)[]} */
  const makeManageesRowFactory = (o, i) => [
    _ => {},
    td => td.textContent = i == null ? o.name : `${i}: ${o.name}`,
    td => td.textContent = `${o.type}`,
    td => td.textContent = `${fmt.hex(o.addr)}`,
    makeViewerNavigatorFactory(o),
  ];

  /** @type {(o: Manager|Managee) => CellFactory} */
  const makeViewerNavigatorFactory = o => td => {
    // remove old button
    const btn0 = td.querySelector('button');
    if (btn0) td.removeChild(btn0);
    // create new button
    const btn = document.createElement('button');
    btn.textContent = '>';
    btn.addEventListener('click', () => {
      fieldsViewer.view(o).catch(e => elmMsg.textContent = e);
    });
    td.appendChild(btn);
  };

  /**************** UI initialization ****************/
  try {
    await client.connect(); // TODO url
    const pid = await api.init();
    console.log('pid:', pid);
    document.body.classList.add('ready');

    await managerList.reload();
  } catch(e) {
    console.log(e);
    showError(e);
  }
});
