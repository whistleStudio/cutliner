import SelectWin from './components/select-win/SelectWin';
import CfgMenu from './components/cfg-menu/CfgMenu';
import './App.css';
import "./assets/base.scss"
import { ConfigProvider } from 'antd';
import { useState } from 'react';



function App () {
  const [isBtnDisabled, setIsBtnDisabled] = useState<boolean>(false);

  const theme = {
    token: {
      colorPrimary: '#eca1a1ff',
      cyan: '#6dad56ff',
    },
  };

  return (
    <ConfigProvider theme={theme}>
      <div className='flex-row-align-center' style={{width: '100%', height: '780px'}}>
        {/* <h1>Hello, World!</h1> */}
        <SelectWin btnState={isBtnDisabled} onChangeBtnState={e => setIsBtnDisabled(e)} />
        <CfgMenu btnState={isBtnDisabled} onChangeBtnState={e => setIsBtnDisabled(e)}/>
      </div>
    </ConfigProvider>
  );
}

export default App;