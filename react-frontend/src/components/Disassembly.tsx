import React, { useState } from 'react';
import '../styles/Disassembly.css';
import { tickClock, getDisassemblyRange, getCurrentProgramCounter } from 'wasm-6502'


function DisassemblyPage() {
    let n_lines = 10;

    let disassemblyObject: Object | null = null;
    let [disassembly, setDisassembly] = useState<Map<number, string> | null>(disassemblyObject ? (new Map(Object.entries(disassemblyObject))) : new Map());
    let [programCounter, setProgramCounter] = useState<number | undefined>(getCurrentProgramCounter())

    console.log("Previous disassembly.");
    disassembly?.forEach((value, key) => {console.log(`${key}: ${value}`)});

    async function onIncrementClock(e: React.MouseEvent<HTMLButtonElement, MouseEvent>) {
        tickClock();

        for (let i = 0; i < 0xFFFF; i += n_lines) {
            let disassemblyRes = JSON.parse(await getDisassemblyRange(i, n_lines));
            let disassemblyObject: Object | null = disassemblyRes ? disassemblyRes : null;
            let new_map = disassemblyObject ? (new Map(Object.entries(disassemblyObject))) : new Map();
            let disassemblyClone = new Map(disassembly);

            new_map.forEach((value, key) => {
                setDisassembly(disassemblyClone?.set(key, value) || disassemblyClone);
            });
        }
        console.log(`Updated disassembly`);
        disassembly?.forEach((value, key) => {console.log(`${key}: ${value}`)});

        setProgramCounter(getCurrentProgramCounter());

    }

    return (
        <div>
            <button onClick={(e) => onIncrementClock(e)}>
                Tick Clock
            </button>
            <div>
                {disassembly ?
                    <div className='DivCenter BoundingBox' style={{ display: "flex", flexDirection: "column", overflow: "scroll" }}>
                        {
                            (() => {
                                let container: [JSX.Element] = [<div> </div>];
                                disassembly.forEach((val, index) => {
                                    container.push(
                                        <div key={index} style={{ backgroundColor: (programCounter === index) ? "blue" : "green" }}>
                                            {val}
                                        </div>)
                                });
                                return container.slice(1);
                            })()
                        }
                    </div>
                    :
                    <div></div>
                }</div>
        </div>

    )
}

export default DisassemblyPage;