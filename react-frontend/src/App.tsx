import { useState } from 'react';
import {create_processor, tick_clock} from 'wasm-6502';

function App() {
   let [processorAvailable, setProcessorAvailable] = useState<boolean>(false);
   let [clock, setClock] = useState<number>(0);
    
   const initializeProcessor = () => {
      if (!processorAvailable) {
         create_processor();
         setProcessorAvailable(true);
      }
   }

   const incrementClock = ()=> {
      if (processorAvailable) {
         console.log("Clock Ticks");
         tick_clock();
         setClock(clock + 1);
      }
   }

  return (
    <div className="App">
      <button onClick={initializeProcessor}> Create Processor</button>
      <button onClick={incrementClock}> Increment Clock</button>
      <label> Clock: {clock} </label>
    </div>
  );
}

export default App;
