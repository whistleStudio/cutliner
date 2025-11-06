import SelectWin from './components/select-win/SelectWin';
import CfgMenu from './components/cfg-menu/CfgMenu';
import './App.css';
import "./assets/base.scss"



function App () {
  return (
    <div className='flex-row-align-center' style={{width: '100%', height: '780px'}}>
      {/* <h1>Hello, World!</h1> */}
      <SelectWin />
      <CfgMenu />
    </div>
  );
}

export default App;