import React, { useEffect, useState, useContext } from 'react'
import './style.scss'
import { withRouter, Link } from 'react-router-dom'
import { ICON_SEARCH, ICON_ARROWBACK, ICON_CLOSE } from '../../Icons'
import { getProfileImageSrcString } from '../../squeakimages/images';
import Loader from '../Loader'
import SqueakCard from '../SqueakCard'

import { Form, Input, Select, Checkbox, Relevant, Debug, TextArea, Option } from 'informed';


import { unwrapResult } from '@reduxjs/toolkit'
import { useDispatch } from 'react-redux'

import SigningProfiles from '../../features/profiles/SigningProfiles'
import ContactProfiles from '../../features/profiles/ContactProfiles'
import {
  setCreateSigningProfile,
  setImportSigningProfile,
  setCreateContactProfile,
  fetchSigningProfiles,
} from '../../features/profiles/profilesSlice'


const Profiles = (props) => {
  const [tab, setTab] = useState('Signing Profiles')
  const [signingProfileModalOpen, setSigningProfileModalOpen] = useState(false)
  const [contactProfileModalOpen, setContactProfileModalOpen] = useState(false)
  const [styleBody, setStyleBody] = useState(false)
  const dispatch = useDispatch()

  const searchOnChange = (param) => {
    if(tab !== 'Search'){setTab('Search')}
    if(param.length>0){
      // TODO: search for a profile by name.
    }
  }

  const toggleSigningProfileModal = (param, type) => {
    setStyleBody(!styleBody)
    setTimeout(()=>{ setSigningProfileModalOpen(!signingProfileModalOpen) },20)
  }

  const toggleContactProfileModal = (param, type) => {
    setStyleBody(!styleBody)
    setTimeout(()=>{ setContactProfileModalOpen(!contactProfileModalOpen) },20)
  }

  const createSigningProfile = ({values}) => {
    console.log(values);
    if (values.importExisting) {
      console.log('Import signing profile with name:', values.name);
      dispatch(setImportSigningProfile({profileName: values.name, privateKey: values.privateKey}))
      .then(unwrapResult)
      .then((pubkey) => {
        console.log('Created profile with pubkey', pubkey);
        dispatch(fetchSigningProfiles());
        props.history.push(`/app/profile/${pubkey}`);
      })
      .catch((err) => {
        alert(err.message);
      });
    } else {
      console.log('Create signing profile with name:', values.name);
      dispatch(setCreateSigningProfile({profileName: values.name}))
      .then(unwrapResult)
      .then((pubkey) => {
        console.log('Created profile with pubkey', pubkey);
        dispatch(fetchSigningProfiles());
        props.history.push(`/app/profile/${pubkey}`);
      })
      .catch((err) => {
        alert(err.message);
      });
    }
    toggleSigningProfileModal();
  }

  const createContactProfile = ({values}) => {
    dispatch(setCreateContactProfile({profileName: values.name, pubkey: values.pubkey}))
    .then(unwrapResult)
    .then((pubkey) => {
      console.log('Created profile with pubkey', pubkey);
      props.history.push(`/app/profile/${pubkey}`);
    })
    .catch((err) => {
      alert(err.message);
    });
    toggleContactProfileModal();
  }

  const handleModalClick = (e) => {
    e.stopPropagation()
  }

  const CreateSigningProfileForm = () => (
    <Form onSubmit={createSigningProfile} className="Squeak-input-side">
      <div className="edit-input-wrap">
        <Input class="informed-input" name="name" label="Profile Name" placeholder="Satoshi" />
      </div>
      <div className="edit-input-wrap">
        <Checkbox class="informed-input" name="importExisting" label="Import Existing Private Key: " />
      </div>
      <Relevant when={({ formState }) => formState.values.importExisting}>
        <div className="edit-input-wrap">
          <Input class="informed-input" name="privateKey" label="Private Key" />
        </div>
      </Relevant>

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

  const CreateContactProfileForm = () => (
    <Form onSubmit={createContactProfile} className="Squeak-input-side">
      <div className="edit-input-wrap">
        <Input class="informed-input" name="name" label="Profile Name" placeholder="Satoshi" />
      </div>
      <div className="edit-input-wrap">
        <Input class="informed-input" name="pubkey" label="Public Key" />
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
    <div>

      <div className="explore-wrapper">
        <div className="explore-header">
          <div className="explore-search-wrapper">
            <div className="explore-search-icon">
              <ICON_SEARCH/>
            </div>
            <div className="explore-search-input">
              <input onChange={(e)=>searchOnChange(e.target.value)} placeholder="Search for people" type="text" name="search"/>
            </div>
          </div>
        </div>
        <div className="profile-details-wrapper">
          <div className="profiles-options">
            <div onClick={(e)=>toggleSigningProfileModal('edit')}
              className='profiles-create-button'>
              <span>Add Signing Profile</span>
            </div>
            <div onClick={(e)=>toggleContactProfileModal('edit')}
              className='profiles-create-button'>
              <span>Add Contact Profile</span>
            </div>
          </div>
        </div>
        <div>
          <div className="explore-nav-menu">
            <div onClick={()=>setTab('Signing Profiles')} className={tab === 'Signing Profiles' ? `explore-nav-item activeTab` : `explore-nav-item`}>
              Signing Profiles
            </div>
            <div onClick={()=>setTab('Contact Profiles')} className={tab === 'Contact Profiles' ? `explore-nav-item activeTab` : `explore-nav-item`}>
              Contact Profiles
            </div>
          </div>
          {tab === 'Signing Profiles' ?
            <SigningProfiles />
            :
            tab === 'Contact Profiles' ?
            <ContactProfiles />
            : <div className="try-searching">
            Nothing to see here ..
            <div/>
            Try searching for people, usernames, or keywords

          </div>
        }
      </div>
    </div>


    {/* Modal for create signing profile */}
    <div onClick={()=>toggleSigningProfileModal()} style={{display: signingProfileModalOpen ? 'block' : 'none'}} className="modal-edit">
      <div onClick={(e)=>handleModalClick(e)} className="modal-content">
        <div className="modal-header">
          <div className="modal-closeIcon">
            <div onClick={()=>toggleSigningProfileModal()} className="modal-closeIcon-wrap">
              <ICON_CLOSE />
            </div>
          </div>
          <p className="modal-title">Add Signing Profile</p>
        </div>

        <div className="modal-body">
          <CreateSigningProfileForm />
        </div>
      </div>
    </div>

    {/* Modal for create contact profile */}
    <div onClick={()=>toggleContactProfileModal()} style={{display: contactProfileModalOpen ? 'block' : 'none'}} className="modal-edit">
      <div onClick={(e)=>handleModalClick(e)} className="modal-content">
        <div className="modal-header">
          <div className="modal-closeIcon">
            <div onClick={()=>toggleContactProfileModal()} className="modal-closeIcon-wrap">
              <ICON_CLOSE />
            </div>
          </div>
          <p className="modal-title">Add Contact Profile</p>
        </div>

        <div className="modal-body">
          <CreateContactProfileForm />
        </div>
      </div>
    </div>


  </div>
)
}

export default withRouter(Profiles)
