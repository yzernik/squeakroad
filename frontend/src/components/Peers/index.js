import React, { useEffect, useState, useContext } from 'react'
import './style.scss'
import { withRouter, Link } from 'react-router-dom'
import { ICON_SEARCH, ICON_ARROWBACK, ICON_CLOSE, ICON_CLIPBOARD } from '../../Icons'
import { getProfileImageSrcString } from '../../squeakimages/images';
import Loader from '../Loader'
import SqueakCard from '../SqueakCard'
import PeerCard from '../../features/peers/PeerCard'
import { CopyToClipboard } from 'react-copy-to-clipboard';
import ReactTooltip from "react-tooltip";

import { Form, Input, Select, Checkbox, Relevant, Debug, TextArea, Option } from 'informed';

import { unwrapResult } from '@reduxjs/toolkit'
import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'

import {
  selectSavedPeers,
  fetchSavedPeers,
  setSavePeer,
} from '../../features/peers/peersSlice'
import {
  fetchExternalAddress,
  selectExternalAddress,
} from '../../features/externalAddress/externalAddressSlice'



const Peers = (props) => {
  const [tab, setTab] = useState('Saved Peers')
  const [savePeerModalOpen, setSavePeerModalOpen] = useState(false)
  const [showExternalAddressModalOpen, setShowExternalAddressModalOpen] = useState(false)
  const [styleBody, setStyleBody] = useState(false)

  const externalAddress = useSelector(selectExternalAddress);
  const peers = useSelector(selectSavedPeers);
  const dispatch = useDispatch();


  useEffect(() => {
    window.scrollTo(0, 0)
    dispatch(fetchExternalAddress());
    dispatch(fetchSavedPeers());
  }, [])

  const toggleSavePeerModal = (param, type) => {
    setStyleBody(!styleBody)
    setTimeout(()=>{ setSavePeerModalOpen(!savePeerModalOpen) },20)
  }

  const toggleShowExternalAddressModalOpen = () => {
    setStyleBody(!styleBody)
    setTimeout(()=>{ setShowExternalAddressModalOpen(!showExternalAddressModalOpen) },20)
  }

  const getNetwork = (useTor) => {
    if (useTor) {
      return 'TORV3';
    }
    return 'IPV4';
  };

  const getExternalAddressStr = () => {
    if (externalAddress) {
      return `${externalAddress.getHost()}:${externalAddress.getPort()}`;
    } else {
      return '';
    }
  };

  function removeHttp(url) {
    return url.replace(/^https?:\/\//, '');
  }

  const savePeer = ({values}) => {
    const network = getNetwork(values.useTor);
    const strippedAddress = removeHttp(values.address);
    const url = new URL(`http://${strippedAddress}`);
    dispatch(setSavePeer({
      name: values.name,
      host: url.hostname,
      port: url.port,
      network: network,
    }));
    toggleSavePeerModal();
  }

  const handleModalClick = (e) => {
    e.stopPropagation()
  }

  const AddPeerForm = () => (
    <Form onSubmit={savePeer} className="Squeak-input-side">
      <div className="edit-input-wrap">
        <p>View a list of public nodes here: <a
          href="https://github.com/squeaknode/nodes/blob/master/README.md"
          target="_blank" rel="noopener noreferrer"
          style={{color: "blue", fontWeight: 'bold'}}
          >https://github.com/squeaknode/nodes/blob/master/README.md</a></p>
      </div>
      <div className="edit-input-wrap">
        <Input class="informed-input" name="name" label="Peer Name (not required)" />
        <Input class="informed-input" name="address" label="Address (host:port)" />
        <Checkbox class="informed-input" name="useTor" label="Connect With Tor: " />
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

  const ShowExternalAddressForm = () => (
    <Form className="Squeak-input-side">
      <div className="edit-input-wrap">
        <p>Other Squeaknode instances can connect to your node using this address to download squeaks and offers.</p>
      </div>
      <div class="float-container">
        <div class="float-child">
          <div className="edit-input-wrap">
            <Input class="informed-input" name="external-address" label="External Address" initialValue={externalAddress && `${getExternalAddressStr()}`} readOnly />
          </div>
        </div>
        <div class="float-child">
          <CopyToClipboard
            text={externalAddress && `${getExternalAddressStr()}`}
            >
            <a data-tip="Copy host">
              <button fullWidth={false}>
                <ICON_CLIPBOARD />
              </button>
            </a>
          </CopyToClipboard>
        </div>
      </div>
      <ReactTooltip effect="solid" />
    </Form>
  );

  return(
    <div>

      <div className="explore-wrapper">
        <div className="peers-header-wrapper">
          <div className="peers-header-content">
            <div className="peers-header-name">
              Peers
            </div>
          </div>
        </div>
        <div className="profile-details-wrapper">
          <div className="profiles-options">
            <div onClick={(e)=>toggleShowExternalAddressModalOpen('edit')}
              className='profiles-create-button'>
              <span>Show External Address</span>
            </div>
            <div onClick={(e)=>toggleSavePeerModal('edit')}
              className='profiles-create-button'>
              <span>Add Peer</span>
            </div>
          </div>
        </div>
        <div>
          <div className="explore-nav-menu">
            <div onClick={()=>setTab('Saved Peers')} className={tab === 'Saved Peers' ? `explore-nav-item activeTab` : `explore-nav-item`}>
              Peers
            </div>
          </div>
          {tab === 'Saved Peers' ?
            peers.map(sp=>{
              return <PeerCard peer={sp}/>
            })
            :
            tab === 'Connected Peers' ?
            <></>
          : <div className="try-searching">
          Nothing to see here ..
          <div/>
          Try searching for people, usernames, or keywords

        </div>
      }
    </div>
  </div>

  {/* Modal for show external address */}
  <div onClick={()=>toggleShowExternalAddressModalOpen()} style={{display: showExternalAddressModalOpen ? 'block' : 'none'}} className="modal-edit">
    <div onClick={(e)=>handleModalClick(e)} className="modal-content">
      <div className="modal-header">
        <div className="modal-closeIcon">
          <div onClick={()=>toggleShowExternalAddressModalOpen()} className="modal-closeIcon-wrap">
            <ICON_CLOSE />
          </div>
        </div>
        <p className="modal-title">Show External Address</p>
      </div>

      <div className="modal-body">
        <ShowExternalAddressForm />
      </div>

    </div>
  </div>


  {/* Modal for create save peer */}
  <div onClick={()=>toggleSavePeerModal()} style={{display: savePeerModalOpen ? 'block' : 'none'}} className="modal-edit">
    <div onClick={(e)=>handleModalClick(e)} className="modal-content">
      <div className="modal-header">
        <div className="modal-closeIcon">
          <div onClick={()=>toggleSavePeerModal()} className="modal-closeIcon-wrap">
            <ICON_CLOSE />
          </div>
        </div>
        <p className="modal-title">Add Peer</p>

      </div>

      <div className="modal-body">
        <AddPeerForm />
      </div>
    </div>
  </div>


</div>
)
}

export default withRouter(Peers)
