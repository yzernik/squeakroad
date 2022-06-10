import React, { useEffect, useState } from 'react'
import { withRouter , Link } from 'react-router-dom'
import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'
import Select from 'react-select'
import moment from 'moment'

import { ICON_ARROWBACK, ICON_HEART, ICON_REPLY, ICON_RETWEET, ICON_HEARTFULL,
  ICON_DELETE, ICON_IMGUPLOAD, ICON_CLOSE, ICON_LOCKFILL } from '../../Icons'

  import { getProfileImageSrcString } from '../../squeakimages/images';

  import SqueakCard from '../../components/SqueakCard'
  import Loader from '../../components/Loader'
  import MakeSqueak from '../squeaks/MakeSqueak'
  import MakeResqueak from '../squeaks/MakeResqueak'
  import DeleteSqueak from '../../features/squeaks/DeleteSqueak'
  import BuySqueak from '../squeaks/BuySqueak'

  import { unwrapResult } from '@reduxjs/toolkit'

  import {
    selectNetwork,
    fetchNetwork,
  } from '../../features/network/networkSlice'
  import {
    selectCurrentSqueak,
    selectCurrentSqueakStatus,
    fetchSqueak,
    clearAll,
    setLikeSqueak,
    setUnlikeSqueak,
    fetchAncestorSqueaks,
    selectAncestorSqueaks,
    selectAncestorSqueaksStatus,
    clearAncestors,
    fetchReplySqueaks,
    selectReplySqueaks,
    selectReplySqueaksStatus,
    clearReplies,
    selectSqueakOffers,
    fetchSqueakOffers,
    setBuySqueak,
    setDownloadSqueak,
  } from './squeaksSlice'
  import {
    fetchPaymentSummaryForSqueak,
    selectPaymentSummaryForSqueak,
  } from '../../features/payments/paymentsSlice'

  import SentPayments from '../../features/payments/SentPayments'
  import ReceivedPayments from '../../features/payments/ReceivedPayments'


  import store from '../../store'


  const Squeak = (props) => {
    const network = useSelector(selectNetwork);
    const currentSqueak = useSelector(selectCurrentSqueak);
    const ancestorSqueaks = useSelector(selectAncestorSqueaks);
    const replySqueaks = useSelector(selectReplySqueaks);
    const loadingCurrentSqueakStatus = useSelector(selectCurrentSqueakStatus)
    const loadingAncestorSqueaksStatus = useSelector(selectAncestorSqueaksStatus)
    const loadingReplySqueaksStatus = useSelector(selectReplySqueaksStatus)
    const paymentSummary = useSelector(selectPaymentSummaryForSqueak)
    const dispatch = useDispatch();

    const [replyModalOpen, setModalOpen] = useState(false)
    const [resqueakModalOpen, setResqueakModalOpen] = useState(false)
    const [deleteModalOpen, setDeleteModalOpen] = useState(false)
    const [buyModalOpen, setBuyModalOpen] = useState(false)
    const [spendingModalOpen, setSpendingModalOpen] = useState(false)
    const [tab, setTab] = useState('Sent Payments')


    const replySqueaksLimit = 10;

    useEffect(() => {
      window.scrollTo(0, 0)
      console.log('useEffect of Squeak component.');
      // dispatch(clearAll());
      dispatch(fetchNetwork());
      dispatch(fetchSqueak(props.id));
      dispatch(fetchAncestorSqueaks(props.id));
      dispatch(fetchReplySqueaks({squeakHash: props.id, limit: 9, lastSqueak: null}));
      dispatch(fetchPaymentSummaryForSqueak({squeakHash: props.id}));
    }, [props.id])

    const toggleReplyModal = (e, type) => {
      if(e){ e.stopPropagation() }
      // if(param === 'edit'){setSaved(false)}
      // if(type === 'parent'){setParent(true)}else{setParent(false)}
      setModalOpen(!replyModalOpen)
    }

    const toggleResqueakModal = (e, type) => {
      if(e){ e.stopPropagation() }

      console.log('Toggling resqueak modal: ', resqueakModalOpen);
      // if(param === 'edit'){setSaved(false)}
      // if(type === 'parent'){setParent(true)}else{setParent(false)}
      setResqueakModalOpen(!resqueakModalOpen)
    }

    const toggleDeleteModal = (e, type) => {
      if(e){ e.stopPropagation() }
      // if(param === 'edit'){setSaved(false)}
      // if(type === 'parent'){setParent(true)}else{setParent(false)}
      setDeleteModalOpen(!deleteModalOpen)
    }

    const toggleBuyModal = () => {
      // load offers on modal open.
      if (!buyModalOpen) {
        console.log('fetchSqueakOffers', props.id);
        dispatch(fetchSqueakOffers(props.id));
      }
      // if(param === 'edit'){setSaved(false)}
      // if(type === 'parent'){setParent(true)}else{setParent(false)}
      setBuyModalOpen(!buyModalOpen)
    }

    const toggleSpendingModal = (param, type) => {
      console.log('Toggle spending modal');
      if(type){setTab(type)}
      setTimeout(()=>{ setSpendingModalOpen(!spendingModalOpen) },20)
    }

    const handleModalClick = (e) => {
      e.stopPropagation()
    }

    const goBack = () => {
      props.history.goBack()
    }

    const resqueak = (id) => {
      alert('Re-Squeak not yet implemented!');
    }

    const unlikeSqueak = (id) => {
      console.log('Clicked like');
      dispatch(setUnlikeSqueak(props.id));
    }

    const likeSqueak = (id) => {
      console.log('Clicked like');
      dispatch(setLikeSqueak(props.id));
    }

    const downloadSqueak = (id) => {
      dispatch(setDownloadSqueak(props.id))
      .then(unwrapResult)
      .then((squeak) => {
        dispatch(fetchAncestorSqueaks(props.id));
      })
      .catch((err) => {
        alert(err.message);
      });
    }

    const getBlockDetailUrl = (blockHash, network) => {
      switch (network) {
        case 'mainnet':
        return `https://blockstream.info/block/${blockHash}`;
        case 'testnet':
        return `https://blockstream.info/testnet/block/${blockHash}`;
        default:
        return '';
      }
    }




    // const ancestorSqueaks = [];

    // const replySqueaks = [];

    // const squeakOffers = [];


    const getRegularSqueakContent = (squeak) => {
      return squeak.getContentStr() ?
        <div className="squeak-content">
          {squeak.getContentStr()}
        </div> :
        <Link>
          <div onClick={()=>toggleBuyModal(props.match.params.id)}
            className="squeak-content locked-content">
            <ICON_LOCKFILL styles={{width:'48px', height:"48px", padding: "5px"}} />
            <div>
              Locked content
            </div>
          </div>
        </Link>
    }

    const getResqueakContent = (squeak) => {
      const resqueakedHash = squeak.getResqueakedHash();
      const resqueakedSqueak = squeak.getResqueakedSqueak();
      const resqueakedAuthor = resqueakedSqueak && resqueakedSqueak.getAuthor();
      return <SqueakCard squeak={resqueakedSqueak} key={resqueakedHash} id={resqueakedHash} user={resqueakedAuthor}
          replies={[]} hasReply={false} />
    }


    const squeak = currentSqueak;
    const author = currentSqueak && currentSqueak.getAuthor();
    const renderedCurrentSqueak =
    <>
    {loadingCurrentSqueakStatus === 'loading' ?
      <Loader /> :
        <div className={squeak ? "squeak-body-wrapper" : "squeak-body-wrapper missing-squeak"}>
          {squeak ?
            <>
            <div className="squeak-header-content">
              <div className="squeak-user-pic">
                <Link to={`/app/profile/${squeak.getAuthorPubkey()}`}>
                  <div className="card-userPic">
                    <img alt="" width="100%" height="49px" src={author ? `${getProfileImageSrcString(author)}` : null}/>
                  </div>
                </Link>
              </div>
              <div className="squeak-user-wrap">
                <Link to={`/app/profile/${squeak.getAuthorPubkey()}`} className="squeak-user-name">
                  {author ?
                    author.getProfileName() :
                    'Unknown Author'
                  }
                </Link>
                <Link to={`/app/profile/${squeak.getAuthorPubkey()}`} className="squeak-username">
                  @{squeak.getAuthorPubkey()}
                </Link>
              </div>
            </div>


            {squeak.getIsResqueak() ?
              getResqueakContent(squeak) :
              getRegularSqueakContent(squeak)
            }


            <div className="squeak-date">
              <a href={getBlockDetailUrl(squeak.getBlockHash(), network)}
                target="_blank"
                rel="noopener noreferrer"
                >
                {moment(squeak.getBlockTime() * 1000).format("h:mm A · MMM D, YYYY")} (Block #{squeak.getBlockHeight()})
              </a>
            </div>
            <div className="squeak-stats">
              <div onClick={()=>toggleSpendingModal('members','Sent Payments')} >
                <div className="int-num"> {paymentSummary && paymentSummary.getAmountSpentMsat() / 1000} </div>
                <div className="int-text"> Sats Spent </div>
              </div>
              <div onClick={()=>toggleSpendingModal('members','Received Payments')} >
                <div className="int-num"> {paymentSummary && paymentSummary.getAmountEarnedMsat() / 1000} </div>
                <div className="int-text"> Sats Earned </div>
              </div>
            </div>
            <div className="squeak-interactions">
              <div onClick={()=>toggleReplyModal()} className="squeak-int-icon">
                <div className="card-icon reply-icon"> <ICON_REPLY /> </div>
                {squeak.getNumReplies()}
              </div>
              <div onClick={()=>toggleResqueakModal()} className="squeak-int-icon">
                <div className="card-icon resqueak-icon">
                  <ICON_RETWEET styles={false ? {stroke: 'rgb(23, 191, 99)'} : {fill:'rgb(101, 119, 134)'}}/>
                </div>
                {squeak.getNumResqueaks()}
              </div>
              <div onClick={()=>{
                  squeak.getLikedTimeMs() ?
                  unlikeSqueak(squeak.getSqueakHash()) :
                  likeSqueak(squeak.getSqueakHash())
                }} className="squeak-int-icon">
                <div className="card-icon heart-icon">
                  {squeak.getLikedTimeMs() ? <ICON_HEARTFULL styles={{fill:'rgb(224, 36, 94)'}}
                  /> : <ICON_HEART/>} </div>
              </div>
              <div onClick={()=>toggleDeleteModal()} className="squeak-int-icon">
                <div className="card-icon delete-icon">
                  <ICON_DELETE styles={{fill:'rgb(101, 119, 134)'}} />
                </div>
              </div>
            </div>
            </> :
            <div className="squeak-header-content">
              <div className="squeak-user-pic">
                <img alt="" style={{borderRadius:'50%', minWidth:'49px'}} width="100%" height="49px" src={null}/>
              </div>
              <div className="squeak-content">
                Missing Squeak
                <div onClick={()=>downloadSqueak(props.match.params.id)}
                  className='profiles-create-button'>
                  <span>Download Squeak</span>
                </div>
              </div>
            </div>
          }
        </div>
      }
      </>

    return <>

    <div className="squeak-wrapper">
      <div className="squeak-header-wrapper">
        <div className="profile-header-back">
          <div onClick={()=>goBack()} className="header-back-wrapper">
            <ICON_ARROWBACK/>
          </div>
        </div>
        <div className="squeak-header-content"> Squeak </div>
      </div>

      {/* Unknown Ancestor squeak */}
      {ancestorSqueaks.length > 0 && ancestorSqueaks[0].getReplyTo() &&
        <SqueakCard squeak={null} key={ancestorSqueaks[0].getReplyTo()} id={ancestorSqueaks[0].getReplyTo()}
          replies={[]} hasReply={true} />
      }

      {/* Ancestor squeaks */}
      {loadingAncestorSqueaksStatus === 'loading' ?
        <Loader /> :
          <>
          {ancestorSqueaks.slice(0, -1).map(r=>{
            // TODO: use replies instead of empty array.
            return <SqueakCard squeak={r} key={r.getSqueakHash()} id={r.getSqueakHash()} user={r.getAuthor()}
              replies={[]} hasReply={true} />
          })}
          </>
      }

      {/* Current squeak */}
      {renderedCurrentSqueak}

      {/* Reply squeaks */}
      {loadingReplySqueaksStatus === 'loading' ?
        <Loader /> :
          <>
          {replySqueaks.map(r=>{
            // TODO: use replies instead of empty array.
            return <SqueakCard squeak={r}  key={r.getSqueakHash()} id={r.getSqueakHash()} user={r.getAuthor()}/>
          })}
          </>
      }

    </div>

    {squeak ?
      <div onClick={()=>toggleReplyModal()} style={{display: replyModalOpen ? 'block' : 'none'}} className="modal-edit">
        {replyModalOpen ?
          <div style={{minHeight: '379px', height: 'initial'}} onClick={(e)=>handleModalClick(e)} className="modal-content">
            <div className="modal-header">
              <div className="modal-closeIcon">
                <div onClick={()=>toggleReplyModal()} className="modal-closeIcon-wrap">
                  <ICON_CLOSE />
                </div>
              </div>
              <p className="modal-title">Reply</p>
            </div>
            <div style={{marginTop:'5px'}} className="modal-body">
              <MakeSqueak replyToSqueak={squeak} submittedCallback={toggleReplyModal} />
            </div>
          </div> : null}
        </div>:null}

        {squeak ?
          <div onClick={()=>toggleResqueakModal()} style={{display: resqueakModalOpen ? 'block' : 'none'}} className="modal-edit">
            {resqueakModalOpen ?
              <div style={{minHeight: '379px', height: 'initial'}} onClick={(e)=>handleModalClick(e)} className="modal-content">
                <div className="modal-header">
                  <div className="modal-closeIcon">
                    <div onClick={()=>toggleResqueakModal()} className="modal-closeIcon-wrap">
                      <ICON_CLOSE />
                    </div>
                  </div>
                  <p className="modal-title">Resqueak</p>
                </div>
                <div style={{marginTop:'5px'}} className="modal-body">
                  <MakeResqueak resqueakedSqueak={squeak} submittedCallback={toggleResqueakModal} />
                </div>
              </div> : null}
            </div>:null}

            {squeak ?
              <div onClick={()=>toggleDeleteModal()} style={{display: deleteModalOpen ? 'block' : 'none'}} className="modal-edit">
                {deleteModalOpen ?
                  <div style={{minHeight: '379px', height: 'initial'}} onClick={(e)=>handleModalClick(e)} className="modal-content">
                    <div className="modal-header">
                      <div className="modal-closeIcon">
                        <div onClick={()=>toggleDeleteModal()} className="modal-closeIcon-wrap">
                          <ICON_CLOSE />
                        </div>
                      </div>
                      <p className="modal-title">Delete Squeak</p>
                    </div>
                    <div style={{marginTop:'5px'}} className="modal-body">
                      <DeleteSqueak squeakHash={squeak.getSqueakHash()} submittedCallback={toggleDeleteModal} />
                    </div>
                  </div> : null}
                </div>:null}

                {squeak ?
                  <div onClick={()=>toggleBuyModal()} style={{display: buyModalOpen ? 'block' : 'none'}} className="modal-edit">
                    {buyModalOpen ?
                      <div style={{minHeight: '379px', height: 'initial'}} onClick={(e)=>handleModalClick(e)} className="modal-content">
                        <div className="modal-header">
                          <div className="modal-closeIcon">
                            <div onClick={()=>toggleBuyModal()} className="modal-closeIcon-wrap">
                              <ICON_CLOSE />
                            </div>
                          </div>
                          <p className="modal-title">Buy Squeak</p>
                        </div>
                        <div style={{marginTop:'5px'}} className="modal-body">
                          <BuySqueak squeak={squeak} submittedCallback={toggleBuyModal} />
                        </div>
                      </div> : null}
                    </div>:null}


                    {/* Modal for sent payments and received payments */}
                    {squeak &&
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
                                <SentPayments squeakHash={props.id} />
                                </>

                              :
                              tab === 'Received Payments' ?
                              <>
                              <ReceivedPayments squeakHash={props.id} />
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
                </div>}


                </>
            }

            export default withRouter(Squeak)
