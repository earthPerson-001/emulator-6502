import React, {useState} from 'react';
import '../styles/InitializePage.css';
import {create_processor as create_6502_processor} from 'wasm-6502';;

function Initialize(): JSX.Element {

    const defaultProcessor = 'none'

    let [processor, setProcessor] = useState<string>(defaultProcessor);
    let [processorLoaded, setProcessorLoaded] = useState<boolean>(false);

    function loadProcessor(id: string) : boolean {
        switch (id) {
            case '6502': {
                if (!processorLoaded){
                    create_6502_processor();
                    setProcessorLoaded(true);
                }
                return true;
            }
            default: {
                return false;
            }
        }
    }

    // when the value of dropdown list changes, change the current processor

    function onDropdownChange(e: React.ChangeEvent<HTMLSelectElement>) {
        selectProcessor(e)
    }

    function selectProcessor( e: React.SyntheticEvent<HTMLSelectElement, Event>): void {
        e.preventDefault();
        setProcessor(e.currentTarget.value);
    }

    return (
        <div className='InitializePage'>
            <section className='DivCenter BoundingBox'>
                <select id="ProcessorDropdown" className='ProcessorDropdown' onChange={onDropdownChange} defaultValue={defaultProcessor}>
                    <option value='6502'>6502</option>
                    <option value='none'>None</option>
                </select>
                <button className='InitializeProcessorButton' onClick={(_) => loadProcessor(processor)}>Initialize Processor</button>
                <label className='ProcessorLoadStatus'>{processorLoaded ? `${processor} Initialized Successfully`:''}</label>
            </section>
        </div>
    )
}

export default Initialize;