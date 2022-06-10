import React , { useEffect, useState, useContext, useRef } from 'react'
import { Link, withRouter, Redirect } from 'react-router-dom'
import './style.scss'
import { ICON_LOGO, ICON_HOME, ICON_HASH, ICON_BELL, ICON_INBOX
, ICON_LIST, ICON_USER, ICON_LAPTOP, ICON_SETTINGS, ICON_HOMEFILL, ICON_HASHFILL,
ICON_BELLFILL, ICON_LISTFILL, ICON_USERFILL, ICON_LAPTOPFILL, ICON_FEATHER,
ICON_CLOSE,ICON_IMGUPLOAD, ICON_INBOXFILL, ICON_LIGHT, ICON_DARK, ICON_TWITTER, ICON_QUESTION } from '../../Icons'
import { ReactComponent as YourSvg } from '../../icon.svg';
import axios from 'axios'
import {API_URL} from '../../config'
import MakeSqueak from '../../features/squeaks/MakeSqueak'
import ContentEditable from 'react-contenteditable'
import Loader from '../Loader'

import { Form, Input, Select, Checkbox, Relevant, Debug, TextArea, Option } from 'informed';

import {
    enable as enableDarkMode,
    disable as disableDarkMode,
    setFetchMethod
} from 'darkreader';

import { unwrapResult } from '@reduxjs/toolkit'
import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'

import {
  fetchSellPrice,
  selectSellPriceDefault,
  selectSellPriceOverride,
  selectSellPriceUsingOverride,
  selectSellPriceInfo,
  setSellPrice,
  setClearSellPrice,
} from '../../features/sellPrice/sellPriceSlice'
import {
  setLogout,
} from '../../features/account/accountSlice'
import {
  selectMakeSqueakStatus,
  selectBuySqueakStatus,
  selectDownloadSqueakStatus,
} from '../../features/squeaks/squeaksSlice'

const Nav = ({history}) => {
    const [moreMenu, setMoreMenu] = useState(false)
    const [theme, setTheme] = useState(true)
    const [modalOpen, setModalOpen] = useState(false)
    const [sellPriceModalOpen, setSellPriceModalOpen] = useState(false)
    const [aboutModalOpen, setAboutModalOpen] = useState(false)
    const [styleBody, setStyleBody] = useState(false)
    const [newSellPriceMsat, setNewSellPriceMsat] = useState(0)

    const session = true;
    const sellPrice = useSelector(selectSellPriceInfo);
    const dispatch = useDispatch();


    const isInitialMount = useRef(true);
    useEffect(() => {
        if (isInitialMount.current){ isInitialMount.current = false }
        else {
            document.getElementsByTagName("body")[0].style.cssText = styleBody && "overflow-y: hidden; margin-right: 17px"
        }
    }, [styleBody])

    useEffect( () => () => document.getElementsByTagName("body")[0].style.cssText = "", [] )

    useEffect(()=>{
        let ran = false
        if(localStorage.getItem('Theme')=='dark'){
            setTheme('dark')
            setFetchMethod(window.fetch)
            enableDarkMode();
        }else if(!localStorage.getItem('Theme')){
            localStorage.setItem('Theme', 'light')
        }
        dispatch(fetchSellPrice());
      }, [])

      const path = history.location.pathname.slice(4)

      const openMore = () => { setMoreMenu(!moreMenu) }

      const handleMenuClick = (e) => { e.stopPropagation() }

    const toggleModal = (e, type) => {
        if(e){ e.stopPropagation() }
        setStyleBody(!styleBody)
        // TODO: Discard content on modal toggle off.
        setTimeout(()=>{ setModalOpen(!modalOpen) },20)
    }

    const toggleSellPriceModal = (param, type) => {
        setMoreMenu(false);
        setStyleBody(!styleBody)
        setTimeout(()=>{ setSellPriceModalOpen(!sellPriceModalOpen) },20)
    }

    const toggleAboutModal = (param, type) => {
        setMoreMenu(false);
        setStyleBody(!styleBody)
        setTimeout(()=>{ setAboutModalOpen(!aboutModalOpen) },20)
    }


    const handleModalClick = (e) => {
        e.stopPropagation()
    }

    const changeTheme = () => {
        setMoreMenu(false);
        if(localStorage.getItem('Theme') === 'dark'){
            disableDarkMode()
            localStorage.setItem('Theme', 'light')
        }else if(localStorage.getItem('Theme') === 'light'){
            localStorage.setItem('Theme', 'dark')
            setFetchMethod(window.fetch)
            enableDarkMode();
        }
    }

    const updateSellPrice = ({values}) => {
        dispatch(setSellPrice(values.sellPriceMsat));
        setNewSellPriceMsat(0);
    }

    const setSellPriceToDefault = () => {
        dispatch(setClearSellPrice());
    }

    const logout = () => {
        dispatch(setLogout());
    }

    const goToTwitterPage = (param, type) => {
      setMoreMenu(false);
      history.push(`/app/twitter`);
    }

    const UpdateSellPriceForm = () => (
      <Form onSubmit={updateSellPrice} className="Squeak-input-side">
        <div className="edit-input-wrap">
          <Input class="informed-input" name="sellPriceMsat" type="number" label="Sell Price (msats)" />
        </div>
        <div className="inner-input-links">
          <div className="input-links-side">
          </div>
          <div className="squeak-btn-holder">
            <div style={{ fontSize: '13px', color: null }}>
            </div>
            <button type="submit" className={'squeak-btn-side squeak-btn-active'}>
              Submit
            </button>
          </div>
        </div>
      </Form>
    );

    return(
        <div className="Nav-component">
        <div className="Nav-width">
            <div className="Nav">
            <div className="Nav-Content">
                <nav className="Nav-wrapper">
                    <Link to={`/app/home`} className="logo-wrapper">
                        <YourSvg styles={{fill:"rgb(29,161,242)", width:'47px', height:"30px"}}/>
                    </Link>
                    <Link className="Nav-link" to={`/app/home`}>
                        <div className={path === '/home' ? "Nav-item-hover active-Nav" : "Nav-item-hover"}>
                            {path === '/home' ? <ICON_HOMEFILL /> : <ICON_HOME />}
                            <div className="Nav-item">Home</div>
                        </div>
                    </Link>
                    <Link to="/app/profiles" className="Nav-link">
                        <div className={path === '/profiles' ? "Nav-item-hover active-Nav" : "Nav-item-hover"}>
                            {path === '/profiles' ? <ICON_USERFILL /> : <ICON_USER />}
                            <div className="Nav-item">Profiles</div>
                        </div>
                    </Link>
                    {session ?
                    <>
                    <Link to="/app/peers" className="Nav-link">
                        <div className={path === '/peers' ? "Nav-item-hover active-Nav" : "Nav-item-hover"}>
                            {path === '/peers' ? <ICON_LAPTOPFILL /> : <ICON_LAPTOP />}
                            <div className="Nav-item">Peers</div>
                        </div>
                    </Link>
                    <Link to="/app/payments" className="Nav-link">
                        <div className={path === '/payments' ? "Nav-item-hover active-Nav" : "Nav-item-hover"}>
                            {path === '/payments' ?  <ICON_LISTFILL /> : <ICON_LIST />}
                            <div className="Nav-item">Payments</div>
                        </div>
                    </Link>
                    <Link to="/app/notifications" className="Nav-link">
                        <div className={path === '/notifications' ? "Nav-item-hover active-Nav" : "Nav-item-hover"}>
                            {path === '/notifications' ? <ICON_BELLFILL /> : <ICON_BELL />}
                            <div className="Nav-item">Notifications</div>
                        </div>
                    </Link>
                    </> : null}
                    <div id="moremenu" onClick={openMore} className="Nav-link">
                        <div className={"Nav-item-hover"}>
                            <ICON_SETTINGS  />
                            <div className="Nav-item">More</div>
                        </div>
                        <div onClick={()=>openMore()} style={{display: moreMenu ? 'block' : 'none'}} className="more-menu-background">
                        <div className="more-modal-wrapper">
                            {moreMenu ?
                            <div style={{top: `${document.getElementById('moremenu').getBoundingClientRect().top - 40}px`, left: `${document.getElementById('moremenu').getBoundingClientRect().left}px`, height: '258px' }} onClick={(e)=>handleMenuClick(e)} className="more-menu-content">
                                    <div onClick={changeTheme} className="more-menu-item">
                                        <span>Change Theme</span>
                                        <span>{theme ? <ICON_DARK/> : <ICON_LIGHT />}</span>
                                    </div>
                                    <div onClick={toggleSellPriceModal} className="more-menu-item">
                                        <span>Update Sell Price</span>
                                        <span><ICON_HASH /></span>
                                    </div>
                                    <div onClick={goToTwitterPage} className="more-menu-item">
                                        <span>Forward Tweets</span>
                                        <span><ICON_TWITTER /></span>
                                    </div>
                                    <div onClick={toggleAboutModal} className="more-menu-item">
                                        <span>About</span>
                                        <span><ICON_QUESTION /></span>
                                    </div>
                                    <div onClick={()=>logout()} className="more-menu-item">
                                        Log out
                                    </div>
                            </div> : null }
                        </div>
                        </div>
                    </div>
                    {session ?
                    <div className="Nav-squeak">
                        <div onClick={()=>toggleModal()} className="Nav-squeak-link">
                            <div className="Nav-squeak-btn btn-hide">
                                Squeak
                            </div>
                            <div className="Nav-squeak-btn btn-show">
                                <span><ICON_FEATHER/></span>
                            </div>
                        </div>
                    </div> : null }
                </nav>
                <div>

                </div>
            </div>
            </div>
        </div>

        <div onClick={()=>toggleModal()} style={{display: modalOpen ? 'block' : 'none'}} className="modal-edit">
            <div style={{minHeight: '270px', height: 'initial'}} onClick={(e)=>handleModalClick(e)} className="modal-content">
                <div className="modal-header">
                    <div className="modal-closeIcon">
                        <div onClick={()=>toggleModal()} className="modal-closeIcon-wrap">
                            <ICON_CLOSE />
                        </div>
                    </div>
                    <p className="modal-title">Squeak</p>
                </div>
                <div style={{marginTop:'5px'}} className="modal-body">
                    <MakeSqueak submittedCallback={toggleModal} />
                </div>
            </div>
        </div>


        {/* Modal for set sell price */}
        <div onClick={()=>toggleSellPriceModal()} style={{display: sellPriceModalOpen ? 'block' : 'none'}} className="modal-edit">
            <div onClick={(e)=>handleModalClick(e)} className="modal-content">
                <div className="modal-header">
                    <div className="modal-closeIcon">
                        <div onClick={()=>toggleSellPriceModal()} className="modal-closeIcon-wrap">
                            <ICON_CLOSE />
                        </div>
                    </div>
                    <p className="modal-title">Sell Price</p>
                    <div className="save-modal-wrapper">
                        <div onClick={setSellPriceToDefault} className="save-modal-btn">
                            Reset To Default
                        </div>
                    </div>
                </div>
                {sellPrice &&
                <div className="modal-body">
                      <div className="edit-input-wrap">
                            Current Sell Price: {sellPrice.getPriceMsat() / 1000} sats
                      </div>
                      <UpdateSellPriceForm />
                </div>
                }
            </div>
        </div>

        {/* Modal for about info */}
        <div onClick={()=>toggleAboutModal()} style={{display: aboutModalOpen ? 'block' : 'none'}} className="modal-edit">
            <div onClick={(e)=>handleModalClick(e)} className="modal-content">
                <div className="modal-header">
                    <div className="modal-closeIcon">
                        <div onClick={()=>toggleAboutModal()} className="modal-closeIcon-wrap">
                            <ICON_CLOSE />
                        </div>
                    </div>
                    <p className="modal-title">About Squeaknode</p>
                </div>
                <div className="modal-body">
                      <div className="edit-input-wrap">
                            Squeaknode is open-source software. View the source code here:&nbsp;
                            <a href="https://github.com/squeaknode/squeaknode"
                              target="_blank" rel="noopener noreferrer"
                              style={{color: "blue", fontWeight: 'bold'}}
                              >https://github.com/squeaknode/squeaknode</a>
                      </div>
                      <div className="edit-input-wrap">
                            Have questions? Join the telegram chat here:&nbsp;
                            <a href="https://t.me/squeaknode"
                              target="_blank" rel="noopener noreferrer"
                              style={{color: "blue", fontWeight: 'bold'}}
                              >t.me/squeaknode</a>
                      </div>
                </div>
            </div>
        </div>

        {/* Block screen with modal when make squeak is waiting. */}
        <div style={{display: useSelector(selectMakeSqueakStatus) === 'loading' ? 'block' : 'none'}} className="modal-edit">
            <div style={{minHeight: '270px', height: 'initial'}} className="modal-content">
                <div className="modal-header">
                    <p className="modal-title">Making squeak...</p>
                </div>
                <div style={{marginTop:'5px'}} className="modal-body">
                    <Loader />
                </div>
            </div>
        </div>

        {/* Block screen with modal when buy squeak is waiting. */}
        <div style={{display: useSelector(selectBuySqueakStatus) === 'loading' ? 'block' : 'none'}} className="modal-edit">
            <div style={{minHeight: '270px', height: 'initial'}} className="modal-content">
                <div className="modal-header">
                    <p className="modal-title">Buying squeak...</p>
                </div>
                <div style={{marginTop:'5px'}} className="modal-body">
                    <Loader />
                </div>
            </div>
        </div>

        {/* Block screen with modal when download squeak is waiting. */}
        <div style={{display: useSelector(selectDownloadSqueakStatus) === 'loading' ? 'block' : 'none'}} className="modal-edit">
            <div style={{minHeight: '270px', height: 'initial'}} className="modal-content">
                <div className="modal-header">
                    <p className="modal-title">Downloading squeak...</p>
                </div>
                <div style={{marginTop:'5px'}} className="modal-body">
                    <Loader />
                </div>
            </div>
        </div>

        </div>
    )
}

export default withRouter(Nav)
