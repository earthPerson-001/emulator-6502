import React, { useCallback, useEffect, useState } from 'react';
import '../styles/OverviewPage.css';
import { tickClock, getRom, getRam, getProcessorStatus, getStorageLayout } from 'wasm-6502'

function OverviewPage() {
    let [ram, setRam] = useState<[number] | null>(JSON.parse(getRam()).mem);
    let [rom, setRom] = useState<[number] | null>(JSON.parse(getRom()).rom);

    let storageLayoutObject: Object | null = JSON.parse(getStorageLayout()) ? JSON.parse(getStorageLayout()) : null
    let [storageLayout, setStorageLayout] = useState<Map<string, number[]> | null>(storageLayoutObject ? (new Map(Object.entries(storageLayoutObject))) : new Map());
    let validStorageRom = storageLayout?.get("secondary_storage");
    let [romStartAndEnd, setRomstartAndEnd] = useState<Array<number> | null>(validStorageRom ? validStorageRom : null);


    // current section of ram and rom
    let [currentRamSection, setCurrentRamSection] = useState<number>(0)
    let [currentRomSection, setCurrentRomSection] = useState<number>(0)

    // max section of ram and rom
    let [maxRamSection, setMaxRamSection] = useState<number | null>(ram?.length ? Math.floor(ram.length / 64) : null)
    let [maxRomSection, setMaxRomSection] = useState<number | null>(rom?.length ? Math.floor(rom.length / 64) : null)


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

        let storageLayoutObject: Object | null = JSON.parse(getStorageLayout()) ? JSON.parse(getStorageLayout()) : null
        setStorageLayout(storageLayoutObject ? (new Map(Object.entries(storageLayoutObject))) : new Map());
        let validStorageRom = storageLayout?.get("secondary_storage");
        setRomstartAndEnd(validStorageRom ? validStorageRom : null);

        let status: number = JSON.parse(getProcessorStatus());
        setCarryFlag((status & (1 << 0)) === (1 << 0));
        setZeroFlag((status & (1 << 1)) === (1 << 1));
        setInterruptDisableFlag((status & (1 << 2)) === (1 << 2));
        setDecimalFlag((status & (1 << 3)) === (1 << 3));
        setBreakFlag((status & (1 << 4)) === (1 << 4));
        setUnusedFlag((status & (1 << 5)) === (1 << 5));
        setOverflowFlag((status & (1 << 6)) === (1 << 6));
        setNegativeFlag((status & (1 << 7)) === (1 << 7));
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
        if (maxSize) { setMaxRamSection(Math.floor(maxSize/256) - 1);}  // 0 to len - 1
    }

    /**
     * Wrapping withing the allowed sections
     * @param section The section to set
     */
    function setCurrentRamSectionWithFilter(section: number) {
        if (section < 0) {
            setCurrentRamSection(0);
        }
        else if ( maxRamSection && section > maxRamSection ) {
            setCurrentRamSection(maxRamSection);
        }
        else {
            setCurrentRamSection(section);
        }
    }

    /**
     * Wrapping withing the allowed sections
     * @param section The section to set
     */
    function setCurrentRomSectionWithFilter(section: number) {
        if (section < 0) {
            setCurrentRomSection(0);
        }
        else if ( maxRomSection && section > maxRomSection ) {
            setCurrentRomSection(maxRomSection);
        }
        else {
            setCurrentRomSection(section);
        }
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
        if (maxSize) { setMaxRomSection(Math.floor(maxSize/256) - 1);}  // 0 to len - 1
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
                        <thead className='TableHead'>
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
                        <tbody className='TableBody'>
                            {
                                romArray?.slice(currentRomSection * 16, currentRomSection * 16 + 17)?.map((value, index) => (
                                    <tr key={index}>
                                        <td key={0}>
                                            {((romStartAndEnd?.at(0) ? romStartAndEnd[0] : 0) + index + currentRomSection * 16).toString(16).padStart(3, '0')}
                                        </td>
                                        {value.map((val, index) => <td style={{opacity: (val + 1)/2}} key={index + 1}>{val.toString(16).padStart(2, '0')} </td>)}
                                    </tr>
                                ))
                            }
                        </tbody>
                    </table>
                    <div className='PreviousAndNextButtons'>
                        <button onClick={(_) => {setCurrentRomSectionWithFilter(currentRomSection - 1)}}>Previous</button>
                        <button onClick={(_) => {setCurrentRomSectionWithFilter(currentRomSection + 1)}}>Next</button>
                    </div>
                </div>
                <div className='RamOverview' onScroll={onRamScroll} style={{ display: "flex", flexDirection: "column" }}>
                    <table className='RamOverviewTable'>
                        <thead className='TableHead'>
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
                        <tbody className='TableBody'>
                            {
                                ramArray?.slice(currentRamSection * 16, currentRamSection * 16 + 17)?.map((value, index) => (
                                    <tr key={index}>
                                        <td key={0}>
                                            {(index + currentRamSection * 16).toString(16).padStart(3, '0')}
                                        </td>
                                        {value.map((val, index) => <td key={index + 1} style={{opacity: (val + 1)/2}}>{val.toString(16).padStart(2, '0')} </td>)}
                                    </tr>
                                ))
                            }
                        </tbody>
                    </table>
                    <div className='PreviousAndNextButtons'>
                        <button onClick={(_) => {setCurrentRamSectionWithFilter(currentRamSection - 1)}}>Previous</button>
                        <button onClick={(_) => {setCurrentRamSectionWithFilter(currentRamSection + 1)}}>Next</button>
                    </div>
                </div>
            </section>

        </div>

    )
}

export default OverviewPage;