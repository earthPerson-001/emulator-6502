import React, { useCallback, useEffect, useRef, useState } from 'react';
import '../styles/OverviewPage.css';
import { tickClock, getRom, getRam, getProcessorStatus } from 'wasm-6502'

function OverviewPage() {
    let [ram, setRam] = useState<[number] | null>(JSON.parse(getRam()).mem);
    let [rom, setRom] = useState<[number] | null>(JSON.parse(getRom()).rom);

    let [ramSize, setRamSize] = useState<number | null>(ram?.length ? ram.length : null);
    let [romSize, setRomSize] = useState<number | null>(rom?.length ? rom.length : null)

    // current section of ram
    let [currentRamSection, setCurrentRamSection] = useState<number>(0)

    // current section of rom
    let [currentRomSection, setCurrentRomSection] = useState<number>(0)


    // table columns
    const nColumns = 16;
    const columnNames = ["   "];  // one column for names
    for (let i: number = 0; i < nColumns; i++) {
        columnNames.push(i.toString(16))
    }

    // cpu status flags
    let [carryFlag, setCarryFlag] = useState<boolean>(false);
    let [zeroFlag, setZeroFlag] = useState<boolean>(false);
    let [interruptDisableFlag, setInterruptDisableFlag] = useState<boolean>(false);
    let [decimalFlag, setDecimalFlag] = useState<boolean>(false);
    let [breakFlag, setBreakFlag] = useState<boolean>(false);
    let [unusedFlag, setUnusedFlag] = useState<boolean>(false);
    let [overflowFlag, setOverflowFlag] = useState<boolean>(false);
    let [negativeFlag, setNegativeFlag] = useState<boolean>(false);


    function increment_clock() {
        tickClock();
        setRam(JSON.parse(getRam()).mem);
        setRom(JSON.parse(getRom()).rom);

        let status: number = JSON.parse(getProcessorStatus());
        setCarryFlag((status & (1 << 0)) === (1 << 0))
        setZeroFlag((status & (1 << 1)) === (1 << 1))
        setInterruptDisableFlag((status & (1 << 2)) === (1 << 2))
        setDecimalFlag((status & (1 << 3)) === (1 << 3))
        setBreakFlag((status & (1 << 4)) === (1 << 4))
        setUnusedFlag((status & (1 << 5)) === (1 << 5))
        setOverflowFlag((status & (1 << 6)) === (1 << 6))
        setNegativeFlag((status & (1 << 7)) === (1 << 7))
    }

    let statusArray = [carryFlag, zeroFlag, interruptDisableFlag, decimalFlag, breakFlag, unusedFlag, overflowFlag, negativeFlag]
    const statusPneumonics = ["C", "Z", "I", "D", "B", "U", "O", "N"];

    let [ramArray, setRamArray] = useState<number[][]>();
    function updateRamArray() {

        let _ramArray = [];
        let maxSize = ram?.length;

        for (let i = 0; ram && maxSize && (i < maxSize); i += 16) {

            _ramArray[Math.floor(i / 16)] = ram?.slice(i, ((i + 16) < maxSize) ? (i + 16) : maxSize);
        }

        setRamArray(_ramArray);
    }

    const updateRamArrayCallback = useCallback(updateRamArray, [ram]);
    const updateRomArrayCallback = useCallback(updateRomArray, [rom]);

    let [romArray, setRomArray] = useState<number[][]>();
    function updateRomArray() {

        let _romArray = [];
        let maxSize = rom?.length;

        for (let i = 0; rom && maxSize && (i < maxSize); i += 16) {
            _romArray[Math.floor(i / 16)] = rom?.slice(i, ((i + 16) < maxSize) ? (i + 16) : maxSize);
        }

        setRomArray(_romArray);
    }

    useEffect(() => {
        updateRamArrayCallback()
    }, [ram, updateRamArrayCallback]);

    useEffect(() => {
        updateRomArrayCallback()
    }, [rom, updateRomArrayCallback])
    

    // on scrolling rom
    function onRomScroll(e: React.UIEvent<HTMLDivElement, UIEvent>) {
        console.log(e)
    }

    // on scrolling ram
    function onRamScroll(e: React.UIEvent<HTMLDivElement, UIEvent>) {
        console.log(e)
    }


    return (
        <div className='OverviewTab DivCenter' >
            <div className='ProcessorSpecificFunctions'>
                <section className='ProcessorStatusOverview'>
                    {
                        statusArray.map((flag, index) => {
                            let labelColor = flag ? "green" : "red";

                            return (
                                <label key={index} style={{ backgroundColor: labelColor }}>
                                    {statusPneumonics[index]}
                                </label>
                            )
                        })
                    }
                </section>

                <button className='TickClockButton' onClick={increment_clock}>Tick Clock</button>
            </div>

            <section className="Storages">
                <div className='RomOverview' onScroll={onRomScroll} style={{ display: "flex", flexDirection: "column" }}>
                    <table className='RomOverviewTable'>
                        <thead>
                            <tr>
                                {
                                    columnNames.map((value, index) => (
                                        <th key={index}>
                                            {value}
                                        </th>
                                    ))
                                }
                            </tr>
                        </thead>
                        <tbody>
                            {
                                romArray?.slice(currentRomSection * 16, currentRomSection * 16 + 17)?.map((value, index) => (
                                    <tr key={index}>
                                        <td key={0}>
                                            {(index + currentRomSection *16).toString(16).padStart(3,'0')}
                                        </td>
                                        {value.map((val, index) => <td key={index + 1}>{val.toString(16).padStart(2,'0')} </td>)}
                                    </tr>
                                ))
                            }
                        </tbody>
                    </table>

                </div>
                <div className='RamOverview' onScroll={onRamScroll} style={{ display: "flex", flexDirection: "column" }}>
                <table className='RamOverviewTable'>
                        <thead>
                            <tr>
                                {
                                    columnNames.map((value, index) => (
                                        <th key={index}>
                                            {value}
                                        </th>
                                    ))
                                }
                            </tr>
                        </thead>
                        <tbody>
                            {
                                ramArray?.slice(currentRamSection * 16, currentRamSection * 16 + 17)?.map((value, index) => (
                                    <tr key={index}>
                                        <td key={0}>
                                            {(index + currentRamSection * 16).toString(16).padStart(3,'0')}
                                        </td>
                                        {value.map((val, index) => <td key={index + 1}>{val.toString(16).padStart(2,'0')} </td>)}
                                    </tr>
                                ))
                            }
                        </tbody>
                    </table>
                </div>
            </section>

        </div>

    )
}

export default OverviewPage;