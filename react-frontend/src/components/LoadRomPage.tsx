import React, { useState } from 'react';
import '../styles/LoadRomPage.css';
import { loadRom, getDefaultProgramCounter } from 'wasm-6502';

function checkText(e: React.ChangeEvent<HTMLTextAreaElement>): boolean {

    let txt = /[g-z]/gi.test(e.target.value);

    if (txt) {
        e.target.value = "";
        e.target.setAttribute("placeholder", "Not a valid hexadecimal number, please input valid hexadecimal numbers");

        return false;
    };

    return true;
}


function LoadRomPage(): JSX.Element {
    let [textBoxValue, setTextBoxValue] = useState<string>("");

    let DEFAULT_PROGRAM_COUNTER_LOCATION = getDefaultProgramCounter();  

    function onTextBoxChange(e: React.ChangeEvent<HTMLTextAreaElement>) {
        if (checkText(e)) {
            setTextBoxValue(e.currentTarget.value);
        }
    }

    function clickFileInput() {
        document.getElementById("give-rom-file-input")?.click();
    }

    return (
        <div className='DivCenter BoundingBox LoadRomPage'>
            <textarea className='TextArea' cols={30} rows={10} value={textBoxValue ? textBoxValue : 0} onChange={onTextBoxChange} />
            <button className='LoadRomButton' onClick={(_) => loadRom(textBoxValue, DEFAULT_PROGRAM_COUNTER_LOCATION)} > Load ROM </button>

            <button onClick={(_) => clickFileInput()}>Upload from file</button>
            <input id="give-rom-file-input" type="file" name="name" style={{display: "none"}} />
        </div>
    )
}

export default LoadRomPage;
