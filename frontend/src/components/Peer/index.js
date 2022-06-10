import React , { useEffect, useState, useContext, useRef } from 'react'
import './style.scss'
import moment from 'moment'
import {  withRouter, Link } from 'react-router-dom'
import Loader from '../Loader'
import SqueakCard from '../SqueakCard'
import {API_URL} from '../../config'
import axios from 'axios'
import {ICON_ARROWBACK, ICON_UPLOAD, ICON_CLOSE,ICON_SEARCH, ICON_SETTINGS } from '../../Icons'

import { Form, Input, Select, Checkbox, Relevant, Debug, TextArea, Option } from 'informed';

import { unwrapResult } from '@reduxjs/toolkit'
import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'

import {
  fetchPeer,
  selectCurrentPeer,
  selectCurrentPeerStatus,
  setSavePeer,
  setDeletePeer,
  setPeerAutoconnectEnabled,
  setPeerAutoconnectDisabled,
  setRenamePeer,
} from '../../features/peers/peersSlice'
import {
  fetchPaymentSummaryForPeer,
  selectPaymentSummaryForPeer,
} from '../../features/payments/paymentsSlice'
import SentPayments from '../../features/payments/SentPayments'
import ReceivedPayments from '../../features/payments/ReceivedPayments'



const Peer = (props) => {

  const [modalOpen, setModalOpen] = useState(false)
  const [moreMenu, setMoreMenu] = useState(false)
  const [editName, setName] = useState('')
  const [editDescription, setDescription] = useState('')
  const [editModalOpen, setEditModalOpen] = useState(false)
  const [deleteModalOpen, setDeleteModalOpen] = useState(false)
  const [savePeerModalOpen, setSavePeerModalOpen] = useState(false)
  const [spendingModalOpen, setSpendingModalOpen] = useState(false)
  const [banner, setBanner] = useState('')
  const [saved, setSaved] = useState(false)
  const [tab, setTab] = useState('Members')
  const [bannerLoading, setBannerLoading] = useState(false)
  const [styleBody, setStyleBody] = useState(false)

  const peer = useSelector(selectCurrentPeer);
  const paymentSummary = useSelector(selectPaymentSummaryForPeer)
  const dispatch = useDispatch();


  useEffect(() => {
    window.scrollTo(0, 0)
    dispatch(fetchPeer({
      network: props.match.params.network,
      host: props.match.params.host,
      port: props.match.params.port,
    }));
    dispatch(fetchPaymentSummaryForPeer({
      network: props.match.params.network,
      host: props.match.params.host,
      port: props.match.params.port,
    }));
  }, [])

  const isInitialMount = useRef(true);
  useEffect(() => {
    if (isInitialMount.current){ isInitialMount.current = false }
    else {
      document.getElementsByTagName("body")[0].style.cssText = styleBody && "overflow-y: hidden; margin-right: 17px"
    }
  }, [styleBody])

  useEffect( () => () => document.getElementsByTagName("body")[0].style.cssText = "", [] )


  const deletePeer = () => {
    let values = {
      peerId: peer.getPeerId(),
    }
    dispatch(setDeletePeer(values));
    toggleDeleteModal();
  }

  function removeHttp(url) {
    return url.replace(/^https?:\/\//, '');
  }

  const savePeer = ({values}) => {
    dispatch(setSavePeer({
      name: values.name,
      host: props.match.params.host,
      port: props.match.params.port,
      network: props.match.params.network,
    }));
    toggleSavePeerModal();
  }

  const enable = (e,id) => {
    e.stopPropagation()
    console.log(id);
    console.log(peer.getPeerId());
    let values = {
      peerId: peer.getPeerId(),
    }

    dispatch(setPeerAutoconnectEnabled(values));
  }

  const disable = (e,id) => {
    e.stopPropagation()
    let values = {
      peerId: peer.getPeerId(),
    }
    dispatch(setPeerAutoconnectDisabled(values));
  }

  const editPeer = ({values}) => {
    console.log('Calling editPeer with name:', values.name);
    dispatch(setRenamePeer({
      peerId: peer.getPeerId(),
      peerName: values.name,
    }));
    setSaved(true)
    toggleEditModal()
  }

  const toggleEditModal = (param, type) => {
    setMoreMenu(false);
    setStyleBody(!styleBody)
    setSaved(false)
    setName(peer.getPeerName())
    setTimeout(()=>{ setEditModalOpen(!editModalOpen) },20)
  }


  const toggleDeleteModal = () => {
    setMoreMenu(false);
    setStyleBody(!styleBody)
    setSaved(false)
    setTimeout(()=>{ setDeleteModalOpen(!deleteModalOpen) },20)
  }

  const toggleSavePeerModal = (param, type) => {
    setMoreMenu(false);
    setStyleBody(!styleBody)
    setTimeout(()=>{ setSavePeerModalOpen(!savePeerModalOpen) },20)
  }

  const toggleSpendingModal = (param, type) => {
    setStyleBody(!styleBody)
    if(type){setTab(type)}
    if(type){setTab(type)}
    setTimeout(()=>{ setSpendingModalOpen(!spendingModalOpen) },20)
  }

  const handleModalClick = (e) => {
    e.stopPropagation()
  }

  const openMore = () => { setMoreMenu(!moreMenu) }

  const handleMenuClick = (e) => { e.stopPropagation() }

  const AddPeerForm = () => (
    <Form onSubmit={savePeer} className="Squeak-input-side">
      <div className="edit-input-wrap">
        <Input class="informed-input" name="name" label="Peer Name (not required)" />
      </div>
      <div className="edit-input-wrap">
        <Input class="informed-input" name="host" label="Host" defaultValue={props.match.params.host} readOnly disabled/>
      </div>
      <div className="edit-input-wrap">
        <Input class="informed-input" name="port" type="number" label="Port" defaultValue={props.match.params.port} readOnly disabled/>
      </div>
      <div className="edit-input-wrap">
        <Checkbox class="informed-input" name="useTor" label="Connect With Tor: " defaultValue={props.match.params.network === "TORV3"} disabled/>
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

  const DeletePeerForm = () => (
    <Form onSubmit={deletePeer} className="Squeak-input-side">
      <div className="inner-input-links">
        <div className="input-links-side">
        </div>
        <div className="squeak-btn-holder">
          <div style={{ fontSize: '13px', color: null }}>
          </div>
          <button type="submit" className={'squeak-btn-side squeak-btn-active'}>
            Delete
          </button>
        </div>
      </div>
    </Form>
  );

  const EditPeerForm = () => (
    <Form onSubmit={editPeer} className="Squeak-input-side">
      <div className="edit-input-wrap">
        <Input class="informed-input" name="name" label="Peer Name" />
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

  console.log(paymentSummary);

  return(
    <div>

      <div className="profile-wrapper">
        <div className="profile-header-wrapper">
          <div className="profile-header-back">
            <div onClick={()=>window.history.back()} className="header-back-wrapper">
              <ICON_ARROWBACK/>
            </div>
          </div>
          <div className="profile-header-content">
            <div className="profile-header-name">
              {props.match.params.host}:{props.match.params.port}
            </div>
          </div>
        </div>
        <div className="profile-details-wrapper">
          <div className="profile-options">

            {peer &&
              <div id="profileMoreMenu" onClick={openMore} className="Nav-link">
                <div className={"Nav-item-hover"}>
                  <ICON_SETTINGS  />
                </div>
                <div onClick={()=>openMore()} style={{display: moreMenu ? 'block' : 'none'}} className="more-menu-background">
                  <div className="more-modal-wrapper">
                    {moreMenu ?
                      <div style={{
                          top: document.getElementById('profileMoreMenu') && `${document.getElementById('profileMoreMenu').getBoundingClientRect().top - 40}px`,
                          left: document.getElementById('profileMoreMenu') && `${document.getElementById('profileMoreMenu').getBoundingClientRect().left}px`,
                          height: '210px',
                        }} onClick={(e)=>handleMenuClick(e)} className="more-menu-content">
                        <div onClick={toggleDeleteModal} className="more-menu-item">
                          <span>Delete Peer</span>
                        </div>
                        <div onClick={toggleEditModal} className="more-menu-item">
                          <span>Edit Peer</span>
                        </div>
                      </div> : null }
                    </div>
                  </div>
                </div>
              }

              {peer &&
                <div onClick={(e)=>
                    peer.getAutoconnect() ?
                    disable(e,peer.getPeerId()) :
                    enable(e,peer.getPeerId())
                  }
                  className={peer.getAutoconnect() ? 'enable-btn-wrap disable-switch' : 'enable-btn-wrap'}>
                  <span><span>{ peer.getAutoconnect() ? 'Enabled' : 'Disabled'}</span></span>
                </div>
              }
            </div>

            <div className="profile-header-content">
              <div className="profile-header-name">
                <div className="profile-name">{peer && peer.getPeerName()}</div>
                <div className="profile-username">Network: {props.match.params.network}</div>
                <div className="profile-username">{props.match.params.host}:{props.match.params.port}</div>
              </div>
            </div>

            <div className="profile-social-box">
              {/* TODO: Implement sats spent */}
              <div onClick={()=>toggleSpendingModal('members','Sent Payments')}>
                <p className="follow-num"> {paymentSummary && paymentSummary.getAmountSpentMsat() / 1000} </p>
                <p className="follow-text"> sats spent </p>
              </div>
              {/* TODO: Implement sats eaned */}
              <div onClick={()=>toggleSpendingModal('members', 'Received Payments')}>
                <p className="follow-num"> {paymentSummary && paymentSummary.getAmountEarnedMsat() / 1000} </p>
                <p className="follow-text"> sats earned </p>
              </div>
            </div>

            <div className="profile-options">
              {!peer &&
                <div onClick={(e)=>toggleSavePeerModal('edit')}
                  className='profiles-create-button'>
                  <span>Add Saved Peer</span>
                </div>
              }
            </div>

          </div>

          <div className="feed-wrapper">
            <div className="feed-trending-card">
              <div className="feed-card-trend">
                <div>Last connection time</div>
                <div>TODO</div>
              </div>
            </div>
          </div>


        </div>

        {/* Modal for delete peer */}
        <div onClick={()=>toggleDeleteModal()} style={{display: deleteModalOpen ? 'block' : 'none'}} className="modal-edit">
          <div onClick={(e)=>handleModalClick(e)} className="modal-content">
            <div className="modal-header">
              <div className="modal-closeIcon">
                <div onClick={()=>toggleDeleteModal()} className="modal-closeIcon-wrap">
                  <ICON_CLOSE />
                </div>
              </div>
              <p className="modal-title">Delete Peer</p>
            </div>
            <div className="modal-body">
              <DeletePeerForm />
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
              <p className="modal-title">Save Peer</p>

            </div>
            <div className="modal-body">
              <AddPeerForm />
            </div>
          </div>
        </div>


        {/* Modal for edit profile */}
        <div onClick={()=>toggleEditModal()} style={{display: editModalOpen ? 'block' : 'none'}} className="modal-edit">
          <div onClick={(e)=>handleModalClick(e)} className="modal-content">
            <div className="modal-header">
              <div className="modal-closeIcon">
                <div onClick={()=>toggleEditModal()} className="modal-closeIcon-wrap">
                  <ICON_CLOSE />
                </div>
              </div>
              <p className="modal-title">Edit Peer</p>
            </div>
            <div className="modal-body">
              <EditPeerForm />
            </div>
          </div>
        </div>


                      {/* Modal for sats spent and earned */}
                      <div onClick={()=>toggleSpendingModal()} style={{display: spendingModalOpen ? 'block' : 'none'}} className="modal-edit">
                        <div onClick={(e)=>handleModalClick(e)} className="modal-content">
                          <div className="modal-header no-b-border">
                            <div className="modal-closeIcon">
                              <div onClick={()=>toggleSpendingModal()} className="modal-closeIcon-wrap">
                                <ICON_CLOSE />
                              </div>
                            </div>
                            <p className="modal-title">{null}</p>
                          </div>
                          <div className="modal-body">
                            <div className="explore-nav-menu">
                              <div onClick={()=>setTab('Sent Payments')} className={tab =='Sent Payments' ? `explore-nav-item activeTab` : `explore-nav-item`}>
                                Sent Payments
                              </div>
                              <div onClick={()=>setTab('Received Payments')} className={tab =='Received Payments' ? `explore-nav-item activeTab` : `explore-nav-item`}>
                                Received Payments
                              </div>
                            </div>
                            <div className="modal-scroll">
                              {tab === 'Sent Payments' ?
                                <>
                                <SentPayments network={props.match.params.network} host={props.match.params.host} port={props.match.params.port} />
                                </>
                              :
                              tab === 'Received Payments' ?
                              <>
                              <ReceivedPayments network={props.match.params.network} host={props.match.params.host} port={props.match.params.port} />
                              </>
                            : <div className="try-searching">
                            Nothing to see here ..
                            <div/>
                            Try searching for people, usernames, or keywords

                          </div>
                        }
                      </div>
                    </div>
                  </div>
                </div>


      </div>
    )
  }

  export default withRouter(Peer)
