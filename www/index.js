// @ts-check
/**
 * @typedef {{addr: number, cls: string, name: string, count: number}} Manager
 * @typedef {{addr: number, cls: string, name: string}} Managee
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
  /** @type {(x: number) => string} */
  float: (LOG_10_2 => x => {
    const u = Math.floor(LOG_10_2*(Math.log2(Math.abs(x))-23));
    return x === 0 ? '0.0' : u > 0 || u < -8 ? x.toExponential(7) :
      x.toFixed(-u);
  })(Math.log10(2)),
};

/**
 * @param {HTMLTableElement} table
 * @param {((td: HTMLTableCellElement)=>void)[][]} gTable
 */
function initTable(table, gTable) {
  const nRow = gTable.length;
  for (let r=table.rows.length; r<nRow; r++) table.insertRow();
  for (let r=table.rows.length; r>nRow; r--) table.deleteRow(-1);
  gTable.forEach((gRow, r) => {
    const row = table.rows[r];
    const nCol = gRow.length;
    for (let c=row.cells.length; c<nCol; c++) row.insertCell();
    for (let c=row.cells.length; c>nCol; c--) row.deleteCell(-1);
    gRow.forEach((g, c) => g(row.cells[c]));
  });
}

document.addEventListener('DOMContentLoaded', async () => {
  const elmMsg = /**@type {HTMLDivElement}*/(document.getElementById('msg'));
  const btnReload = /**@type {HTMLButtonElement}*/(document.getElementById('btn-reload'));
  const elmManagers = /**@type {HTMLTableElement}*/(document.getElementById('managers'));
  const elmFieldsViewer = /**@type {HTMLTableElement}*/(document.getElementById('fields-viewer'));

  const client = Client({
    onClose() {
      elmMsg.textContent = `Disconnected from server. Please reload the page.`;
      document.body.classList.add('disconnected');
    },
  });
  const {api} = client;
  Object.assign(window, {client, api}); // TODO

  /**************** UI definition ****************/
  /**
   * @param {HTMLTableElement} elm
   */
  function FieldsViewer(elm) {
    const tdidxVal = 2;
    let hAnm = NaN;
    let t0 = 0;
    /** @type {Managee|null} */
    let target = null;
    async function readValues() {
      if (target == null) return [];
      const values = await api.read([target.addr], target.cls);
      return values instanceof Array ? values : [values];
    }
    /** @param {DOMHighResTimeStamp} t */
    async function render(t) {
      if (t-t0 >= 33) { // TODO configurable fps
        (await readValues())
          .forEach((s, i) => elm.rows[i].cells[tdidxVal].textContent = s);
        t0 = t;
      }
      hAnm = requestAnimationFrame(render);
    }
    const methods = {
      reload() {
        api.reload().then(() => {
          target != null && methods.view(target);
        }, err => {
          elmMsg.textContent = err;
        });
      },
      /** @param {Manager|Managee} o */
      async view(o) {
        btnReload.classList.remove('hidden');
        cancelAnimationFrame(hAnm);
        target = o;
        const fields = await api.getFields(o.cls);
        const values = readValues();
        initTable(elm, fields.map((r, i) => [
          td => td.textContent = r[0],
          td => td.textContent = r[1],
          td => td.textContent = values[i],
          // td => td.textContent = r[2], // TODO
          td => td.textContent = r[3],
          td => td.textContent = r[4],
        ]));
        if (fields.length) hAnm = requestAnimationFrame(render);
      },
    }
    return methods;
  }
  const fieldsViewer = FieldsViewer(elmFieldsViewer);
  btnReload.addEventListener('click', () => {
    fieldsViewer.reload();
  });

  /** @type {(o: Manager) => RowFactory} */
  const makeManagersRowFactory = o => [
    td => {
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
    td => td.textContent = `${o.cls}`,
    td => td.textContent = `${fmt.hex(o.addr)}`,
    makeViewerNavigatorFactory(o),
  ];

  /** @type {(o: Managee, i: number) => ((td: HTMLTableCellElement) => void)[]} */
  const makeManageesRowFactory = (o, i) => [
    _ => {},
    td => td.textContent = `${i}: (${o.name})`,
    td => td.textContent = `${o.cls}`,
    td => td.textContent = `${fmt.hex(o.addr)}`,
    makeViewerNavigatorFactory(o),
  ];

  /** @type {(o: Manager|Managee) => CellFactory} */
  const makeViewerNavigatorFactory = o => td => {
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

    const managers = await api.getManagers();
    initTable(elmManagers, managers.map(makeManagersRowFactory));
  } catch(e) {
    elmMsg.textContent = e;
    return; // TODO
  }
});
