import React from 'react';
import SelectWin from './components/select-win/SelectWin';
import CfgMenu from './components/cfg-menu/CfgMenu';
import './App.css';
import "./assets/base.scss"

const SharedImgDataContext = React.createContext({
  imgData: null as Uint8Array | null,
  setImgData: (data: Uint8Array | null) => {},
}); 

function App () {
  return (
    <div className='flex-row-align-center' style={{width: '100%', height: '780px'}}>
      {/* <h1>Hello, World!</h1> */}
      <SharedImgDataContext.Provider value={{
        imgData: null,
        setImgData: () => {},
      }}>
        <SelectWin />
        <CfgMenu />
      </SharedImgDataContext.Provider>
    </div>
  );
}

export default App;