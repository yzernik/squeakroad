import React , { useEffect, useState, useContext, useRef} from 'react'
import './style.scss'
import { ICON_ARROWBACK, ICON_MARKDOWN, ICON_DATE, ICON_CLOSE, ICON_UPLOAD, ICON_NEWMSG, ICON_SETTINGS, ICON_DARK } from '../../Icons'
import { withRouter, Link } from 'react-router-dom'
import { getProfileImageSrcString } from '../../squeakimages/images';
import Loader from '../Loader'
import moment from 'moment'
import SqueakCard from '../SqueakCard'
import {API_URL} from '../../config'

import { Form, Input, Select, Checkbox, Relevant, Debug, TextArea, Option } from 'informed';


import { unwrapResult } from '@reduxjs/toolkit'
import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'

import {
  fetchProfile,
  setFollowProfile,
  setUnfollowProfile,
  selectCurrentProfile,
  setDeleteProfile,
  setRenameProfile,
  setCreateContactProfile,
  setProfileImage,
  setClearProfileImage,
  getProfilePrivateKey,
} from '../../features/profiles/profilesSlice'
import {
  fetchProfileSqueaks,
  selectProfileSqueaks,
  selectLastProfileSqueak,
  selectProfileSqueaksStatus,
  clearProfileSqueaks,
} from '../../features/squeaks/squeaksSlice'
import {
  fetchPaymentSummaryForPubkey,
  selectPaymentSummaryForPubkey,
} from '../../features/payments/paymentsSlice'

import SentPayments from '../../features/payments/SentPayments'
import ReceivedPayments from '../../features/payments/ReceivedPayments'



const Profile = (props) => {
  const [activeTab, setActiveTab] = useState('Squeaks')
  const [moreMenu, setMoreMenu] = useState(false)
  const [privateKey, setPrivateKey] = useState('')
  const [editModalOpen, setEditModalOpen] = useState(false)
  const [changeImageModalOpen, setChangeImageModalOpen] = useState(false)
  const [deleteModalOpen, setDeleteModalOpen] = useState(false)
  const [exportModalOpen, setExportModalOpen] = useState(false)
  const [spendingModalOpen, setSpendingModalOpen] = useState(false)
  const [createModalOpen, setCreateModalOpen] = useState(false)
  const [tab, setTab] = useState('Sent Payments')
  const [styleBody, setStyleBody] = useState(false)
  const userParam = props.match.params.username

  const profileSqueaks = useSelector(selectProfileSqueaks);
  const lastUserSqueak = useSelector(selectLastProfileSqueak);
  const profileSqueaksStatus = useSelector(selectProfileSqueaksStatus);
  // const privateKey = 'TODO';
  const paymentSummary = useSelector(selectPaymentSummaryForPubkey)

  const user = useSelector(selectCurrentProfile);
  const dispatch = useDispatch();



  useEffect(() => {
    window.scrollTo(0, 0)
    dispatch(fetchProfile(props.match.params.username));
    reloadSqueaks();
    dispatch(fetchPaymentSummaryForPubkey({pubkey: props.match.params.username}));
    //preventing edit modal from apprearing after clicking a user on memOpen
    setEditModalOpen(false);
  }, [props.match.params.username])

  const isInitialMount = useRef(true);
  useEffect(() => {
    if (isInitialMount.current){ isInitialMount.current = false }
    else {
      document.getElementsByTagName("body")[0].style.cssText = styleBody && "overflow-y: hidden; margin-right: 17px"
    }
  }, [styleBody])

  useEffect( () => () => document.getElementsByTagName("body")[0].style.cssText = "", [] )

  const changeTab = (tab) => {
    setActiveTab(tab)
  }

  const editProfile = ({values}) => {
    dispatch(setRenameProfile({
      profileId: user.getProfileId(),
      profileName: values.name,
    }));
    // TODO: chain action to update profile squeaks with the new name.
    toggleEditModal()
  }

  const deleteProfile = () => {
    let values = {
      profileId: user.getProfileId(),
    }
    console.log('Delete user here');
    dispatch(setDeleteProfile(values));
    toggleDeleteModal();
  }


  const exportPrivateKey = () => {
    dispatch(getProfilePrivateKey({
      profileId: user.getProfileId(),
    }))
    .then(unwrapResult)
    .then((privateKey) =>{
      console.log(privateKey);
      setPrivateKey(privateKey);
    });
  }

  const createContactProfile = ({values}) => {
    dispatch(setCreateContactProfile({
      pubkey: userParam,
      profileName: values.name,
    }))
    .then(() => {
      dispatch(fetchProfile(props.match.params.username));
    });
    toggleCreateModal();
  }

  const toggleEditModal = (param, type) => {
    setMoreMenu(false);
    setStyleBody(!styleBody)
    setTimeout(()=>{ setEditModalOpen(!editModalOpen) },20)
  }

  const toggleChangeImageModal = (param, type) => {
    setMoreMenu(false);
    setStyleBody(!styleBody)
    setTimeout(()=>{ setChangeImageModalOpen(!changeImageModalOpen) },20)
  }

  const toggleDeleteModal = () => {
    setMoreMenu(false);
    setStyleBody(!styleBody)
    setTimeout(()=>{ setDeleteModalOpen(!deleteModalOpen) },20)
  }

  const toggleExportModal = () => {
    setMoreMenu(false);
    setStyleBody(!styleBody)
    setTimeout(()=>{ setExportModalOpen(!exportModalOpen) },20)
  }

  const toggleSpendingModal = (param, type) => {
    setStyleBody(!styleBody)
    if(type){setTab(type)}
    if(type){setTab(type)}
    setTimeout(()=>{ setSpendingModalOpen(!spendingModalOpen) },20)
  }

  const toggleCreateModal = (param, type) => {
    setStyleBody(!styleBody)
    setTimeout(()=>{ setCreateModalOpen(!createModalOpen) },20)
  }

  const handleModalClick = (e) => {
    e.stopPropagation()
  }

  const followUser = (e,id) => {
    e.stopPropagation()
    dispatch(setFollowProfile(id));
  }

  const unfollowUser = (e,id) => {
    console.log(e);
    e.stopPropagation()
    dispatch(setUnfollowProfile(id));
  }

  const changeAvatar = () => {
    let file = document.getElementById('avatar').files[0];
    uploadAvatar(file);
    toggleChangeImageModal();
  }

  const clearAvatar = () => {
    dispatch(setClearProfileImage({
      profileId: user.getProfileId(),
    }));
  }

  const uploadAvatar = (file) => {
    if (file == null)
    return;
    const reader = new FileReader();
    reader.addEventListener('load', () => {
      // convert image file to base64 string
      // preview.src = reader.result;
      const imageBase64Stripped = reader.result.split(',')[1];
      uploadAvatarAsBase64(imageBase64Stripped);
    }, false);
    if (file) {
      reader.readAsDataURL(file);
    }
  };

  const uploadAvatarAsBase64 = (imageBase64) => {
    dispatch(setProfileImage({
      profileId: user.getProfileId(),
      imageBase64: imageBase64,
    }));
  };

  const getLastSqueak = (squeakLst) => {
    if (squeakLst == null) {
      return null;
    } if (squeakLst.length === 0) {
      return null;
    }
    return squeakLst.slice(-1)[0];
  };

  const getMoreSqueaks = () => {
    let lastSqueak = getLastSqueak(profileSqueaks);
    dispatch(fetchProfileSqueaks({
      profilePubkey: props.match.params.username,
      limit: 10,
      lastSqueak: lastUserSqueak,
    }));
  }

  const reloadSqueaks = () => {
    dispatch(clearProfileSqueaks());
    dispatch(fetchProfileSqueaks({
      profilePubkey: props.match.params.username,
      limit: 10,
      lastSqueak: null,
    }));
  }

  const openMore = () => { setMoreMenu(!moreMenu) }

  const handleMenuClick = (e) => { e.stopPropagation() }


  const AddContactProfileForm = () => (
    <Form onSubmit={createContactProfile} className="Squeak-input-side">
      <div className="edit-input-wrap">
        <Input class="informed-input" name="name" label="Profile Name" placeholder="Satoshi" />
      </div>
      <div className="edit-input-wrap">
        <Input class="informed-input" name="pubkey" label="Public Key" defaultValue={userParam} readOnly disabled />
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

  const EditProfileForm = () => (
    <Form onSubmit={editProfile} className="Squeak-input-side">
      <div className="edit-input-wrap">
        <Input class="informed-input" name="name" label="Profile Name" placeholder="Satoshi" />
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

  const ChangeProfileImageForm = () => (
    <Form onSubmit={uploadAvatar} className="Squeak-input-side">
      <div className="modal-profile-pic">
        <div className="modal-back-pic">
          <img src={user ? `${getProfileImageSrcString(user)}` : null} alt="profile" />
          <div>
            <ICON_UPLOAD/>
            <input onChange={()=>changeAvatar()} title=" " id="avatar" style={{opacity:'0'}} type="file"/>
          </div>
        </div>
      </div>
    </Form>
  );


  const DeleteProfileForm = () => (
    <Form onSubmit={deleteProfile} className="Squeak-input-side">
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

  const ExportPrivateKeyForm = () => (
    <Form onSubmit={exportPrivateKey} className="Squeak-input-side">
      <div className="edit-input-wrap">
        <Input class="informed-input" name="privateKey" label="Display Private Key" initialValue={privateKey} readOnly />
      </div>
      <div className="inner-input-links">
        <div className="input-links-side">
        </div>
        <div className="squeak-btn-holder">
          <div style={{ fontSize: '13px', color: null }}>
          </div>
          <button type="submit" className={'squeak-btn-side squeak-btn-active'}>
            Export
          </button>
        </div>
      </div>
    </Form>
  );

  console.log(paymentSummary);

  return(
    <div>
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
                {userParam}
              </div>
            </div>
          </div>
          <div className="profile-banner-wrapper">
            <img alt=""/>
          </div>
          <div className="profile-details-wrapper">
            <div className="profile-options">
              <div className="profile-image-wrapper">
                <img src={user ? `${getProfileImageSrcString(user)}` : null} alt=""/>
              </div>

              {user &&
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
                          <div onClick={toggleEditModal} className="more-menu-item">
                            <span>Edit Profile</span>
                          </div>
                          <div onClick={toggleChangeImageModal} className="more-menu-item">
                            <span>Change Image</span>
                          </div>
                          {user.getHasPrivateKey() &&
                            <div onClick={toggleExportModal} className="more-menu-item">
                              <span>Export Private Key</span>
                            </div>
                          }
                          <div onClick={toggleDeleteModal} className="more-menu-item">
                            <span>Delete Profile</span>
                          </div>
                        </div> : null }
                      </div>
                    </div>
                  </div>
                }

                {user &&
                  <div onClick={(e)=>
                      user.getFollowing() ?
                      unfollowUser(e,user.getProfileId()) :
                      followUser(e,user.getProfileId())
                    }
                    className={user.getFollowing() ? 'unfollow-switch profile-edit-button' : 'profile-edit-button'}>
                    <span><span>{ user.getFollowing() ? 'Following' : 'Follow'}</span></span>
                  </div>
                }

                {!user &&
                  <div onClick={(e)=>toggleCreateModal('create')}
                    className='profiles-create-button'>
                    <span>Add Contact Profile</span>
                  </div>
                }
              </div>
              <div className="profile-details-box">
                <div className="profile-name">{user ? user.getProfileName() : ''}</div>
                <div className="profile-username">@{userParam}</div>
                <div className="profile-info-box">
                  &nbsp;
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
            </div>
            <div className="profile-nav-menu">
              <div key={'squeaks'} onClick={()=>changeTab('Squeaks')} className={activeTab ==='Squeaks' ? `profile-nav-item activeTab` : `profile-nav-item`}>
                Squeaks
              </div>
              <div key={'replies'} onClick={()=>changeTab('Squeaks&Replies')} className={activeTab ==='Squeaks&Replies' ? `profile-nav-item activeTab` : `profile-nav-item`}>
                Squeaks & replies
              </div>
              <div key={'liked'} onClick={()=>changeTab('Liked')} className={activeTab ==='Liked' ? `profile-nav-item activeTab` : `profile-nav-item`}>
                Liked
              </div>
            </div>
            {activeTab === 'Squeaks' ?
              profileSqueaks.map(t=>{
                if(!t.getReplyTo())
                return <SqueakCard squeak={t} key={t.getSqueakHash()} id={t.getSqueakHash()} user={t.getAuthor()} />
              }) :
              activeTab === 'Squeaks&Replies' ?
              profileSqueaks.map(t=>{
                return <SqueakCard squeak={t} key={t.getSqueakHash()} id={t.getSqueakHash()} user={t.getAuthor()} />
              }) :
              activeTab === 'Liked' ?
              profileSqueaks.map(t=>{
                if(t.getLikedTimeMs())
                return <SqueakCard squeak={t} key={t.getSqueakHash()} id={t.getSqueakHash()} user={t.getAuthor()} />
              }) :
              null}
              {/* TODO: fix get loading state by doing this: https://medium.com/stashaway-engineering/react-redux-tips-better-way-to-handle-loading-flags-in-your-reducers-afda42a804c6 */}
              {profileSqueaks.length > 0 &&
                <>
                {profileSqueaksStatus == 'loading' ?
                  <Loader /> :
                    <div onClick={() => getMoreSqueaks()} className='squeak-btn-side squeak-btn-active'>
                      Load more
                    </div>}
                    </>
                }

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
                    <p className="modal-title">Edit Profile</p>
                  </div>
                  <div className="modal-body">
                    <EditProfileForm />
                  </div>
                </div>
              </div>

              {/* Modal for change profile image */}
              <div onClick={()=>toggleChangeImageModal()} style={{display: changeImageModalOpen ? 'block' : 'none'}} className="modal-edit">
                <div onClick={(e)=>handleModalClick(e)} className="modal-content">
                  <div className="modal-header">
                    <div className="modal-closeIcon">
                      <div onClick={()=>toggleChangeImageModal()} className="modal-closeIcon-wrap">
                        <ICON_CLOSE />
                      </div>
                    </div>
                    <p className="modal-title">Change Profile Image</p>
                  </div>
                  <div className="modal-body">
                    <div className="modal-banner">
                    </div>
                    <ChangeProfileImageForm />
                  </div>
                </div>
              </div>


              {/* Modal for delete profile */}
              <div onClick={()=>toggleDeleteModal()} style={{display: deleteModalOpen ? 'block' : 'none'}} className="modal-edit">
                <div onClick={(e)=>handleModalClick(e)} className="modal-content">
                  <div className="modal-header">
                    <div className="modal-closeIcon">
                      <div onClick={()=>toggleDeleteModal()} className="modal-closeIcon-wrap">
                        <ICON_CLOSE />
                      </div>
                    </div>
                    <p className="modal-title">Delete Profile</p>
                  </div>

                  <div className="modal-body">
                    <DeleteProfileForm />
                  </div>
                </div>
              </div>

              {/* Modal for export profile */}
              <div onClick={()=>toggleExportModal()} style={{display: exportModalOpen ? 'block' : 'none'}} className="modal-edit">
                <div onClick={(e)=>handleModalClick(e)} className="modal-content">
                  <div className="modal-header">
                    <div className="modal-closeIcon">
                      <div onClick={()=>toggleExportModal()} className="modal-closeIcon-wrap">
                        <ICON_CLOSE />
                      </div>
                    </div>
                    <p className="modal-title">Export Private Key</p>
                  </div>

                  <div className="modal-body">
                    <ExportPrivateKeyForm />
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
                        <SentPayments pubkey={props.match.params.username} />
                        </>

                      :
                      tab === 'Received Payments' ?
                      <>
                      <ReceivedPayments pubkey={props.match.params.username} />
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

        {/* Modal for create contact profile */}
        <div onClick={()=>toggleCreateModal()} style={{display: createModalOpen ? 'block' : 'none'}} className="modal-edit">
          <div onClick={(e)=>handleModalClick(e)} className="modal-content">
            <div className="modal-header">
              <div className="modal-closeIcon">
                <div onClick={()=>toggleCreateModal()} className="modal-closeIcon-wrap">
                  <ICON_CLOSE />
                </div>
              </div>
              <p className="modal-title">Add Contact Profile</p>
            </div>

            <div className="modal-body">
              <AddContactProfileForm />
            </div>
          </div>
        </div>


      </div>
    </div>
  )
}

export default withRouter(Profile)
