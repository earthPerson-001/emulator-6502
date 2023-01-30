import { useState } from 'react';
import LoadRomPage from './components/LoadRomPage';
import Overview from './components/OverviewPage';
import Disassembly from './components/Disassembly';
import Initialize from './components/InitializePage';

import './App.css'

const INITIALIZE_PAGE = 0
const LOAD_ROM_PAGE = 1;
const OVERVIEW_PAGE = 2;
const DISASSEMBLY_PAGE = 3;

function App() {
   // state representing the currently active tab
   let [tabNumber, setTabNumber] = useState<number>(INITIALIZE_PAGE);

   // buttons to change tabs

   const pages: string[] = ["Initialize", "Load Rom", "Overview", "Disassembly"];
   const pageNumbers: number[] = [INITIALIZE_PAGE, LOAD_ROM_PAGE, OVERVIEW_PAGE, DISASSEMBLY_PAGE];

   const tabButtons: JSX.Element[] = pageNumbers.map((pageNumber) => {
      // change the selected tab button color
      let button_style = (pageNumber===tabNumber) ? {backgroundColor: "green"} : {};

      return (<button key={pageNumber} className='TabButton' style={button_style}  onClick={(_) => { setTabNumber(pageNumber) }}>
         {pages[pageNumber]}
      </button>)
   }
   );

   return (
      <div className="App">
         {/* tab selection */}
         <div className="TabSelector">
            {tabButtons}
         </div>

         {/* Tab like behaviour depending upon the tabNumber */}
         <div className="Tab">
            {(() => {
               switch (tabNumber) {
                  case INITIALIZE_PAGE: return <Initialize />
                  case LOAD_ROM_PAGE: return <LoadRomPage />;
                  case OVERVIEW_PAGE: return <Overview />;
                  case DISASSEMBLY_PAGE: return <Disassembly />;
                  default: return <Initialize />;
               }
            })()}
         </div>
      </div>
   );
}

export default App;
