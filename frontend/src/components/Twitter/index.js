import React, { useEffect, useState, useContext } from 'react'
import './style.scss'
import { withRouter, Link } from 'react-router-dom'
import { ICON_SEARCH, ICON_ARROWBACK, ICON_CLOSE } from '../../Icons'
import { getProfileImageSrcString } from '../../squeakimages/images';
import Loader from '../Loader'
import SqueakCard from '../SqueakCard'

import { Form, Input, Checkbox, Select, Relevant, Debug, TextArea, Option } from 'informed';

import { unwrapResult } from '@reduxjs/toolkit'
import { useDispatch } from 'react-redux'
import { useSelector } from 'react-redux'

import TwitterAccounts from '../../features/twitterAccounts/TwitterAccounts'
import {
  setCreateSigningProfile,
} from '../../features/profiles/profilesSlice'

import {
  setCreateTwitterAccount,
} from '../../features/twitterAccounts/twitterAccountsSlice'

import {
  selectSigningProfiles,
  fetchSigningProfiles,
} from '../../features/profiles/profilesSlice'


const Twitter = (props) => {
  const [tab, setTab] = useState('Signing Profiles')
  const [twitterAccountModalOpen, setTwitterAccountModalOpen] = useState(false)
  const [styleBody, setStyleBody] = useState(false)

  const signingProfiles = useSelector(selectSigningProfiles);

  const dispatch = useDispatch()

  const searchOnChange = (param) => {
    if(tab !== 'Search'){setTab('Search')}
    if(param.length>0){
      // TODO: search for a profile by name.
    }
  }

  const optionsFromProfiles = (profiles) => {
    return profiles.map((p) => {
        return { value: p, label: p.getProfileName() }
        // return { value: 'chocolate', label: 'Chocolate' }
      });
  }

  const toggleCreateTwitterAccountModal = (param, type) => {
    setStyleBody(!styleBody)
    setTimeout(()=>{ setTwitterAccountModalOpen(!twitterAccountModalOpen) },20)
  }

  const createTwitterAccount = ({values}) => {
    console.log('Create twitter account with handle:', values.twitterHandle);
    if (!values.signingProfileId) {
      alert('Signing profile must be set.');
      return;
    }
    dispatch(setCreateTwitterAccount({
      twitterHandle: values.twitterHandle,
      profileId: values.signingProfileId,
      bearerToken: values.twitterBearerToken,
    }))
    .then(unwrapResult)
    .then((pubkey) => {
      console.log('Created twitter account with handle', values.twitterHandle);
    })
    .catch((err) => {
      alert(err.message);
    });
    toggleCreateTwitterAccountModal();
  }

  const handleModalClick = (e) => {
    e.stopPropagation()
  }

  const AddTwitterAccountForm = () => (
    <Form onSubmit={createTwitterAccount} className="Squeak-input-side">
      <div className="edit-input-wrap">
        <Input class="informed-input" name="twitterHandle" label="Twitter Handle" />
      </div>
      <div className="edit-input-wrap">
        <Input class="informed-input" name="twitterBearerToken" label="Twitter Bearer Token" />
      </div>
      <div className="edit-input-wrap">
      <Select class="informed-select" name="signingProfileId" label="Signing Profile" initialValue="">
        <Option value="" disabled>
          Select Signing Profile...
        </Option>
        {signingProfiles.map(p => {
          return <option value={p.getProfileId()}>{p.getProfileName()}</option>
        })}
      </Select>
      </div>
      <div className="inner-input-links">
        <div className="input-links-side">
        </div>
        <div className="squeak-btn-holder">
          <div style={{ fontSize: '13px', color: null }}>
          </div>
          <button type="submit" className={'squeak-btn-side squeak-btn-active'}>
            Add Twitter Account
          </button>
        </div>
      </div>
    </Form>
  );

  return(
    <div>

      <div className="explore-wrapper">
        <div className="peers-header-wrapper">
            <div className="peers-header-content">
                <div className="peers-header-name">
                    Twitter Accounts
                </div>
            </div>
        </div>
        <div className="profile-details-wrapper">
          <div className="profiles-options">
            <div onClick={(e)=>toggleCreateTwitterAccountModal('edit')}
              className='profiles-create-button'>
              <span>Add Twitter Account</span>
            </div>
          </div>
        </div>
        <div>
          <div className="explore-nav-menu">
            <div onClick={()=>setTab('Signing Profiles')} className={tab === 'Signing Profiles' ? `explore-nav-item activeTab` : `explore-nav-item`}>
              Twitter Accounts
            </div>
          </div>
          <TwitterAccounts />
        </div>
      </div>


      {/* Modal for create signing profile */}
      <div onClick={()=>toggleCreateTwitterAccountModal()} style={{display: twitterAccountModalOpen ? 'block' : 'none'}} className="modal-edit">
        <div onClick={(e)=>handleModalClick(e)} className="modal-content">
          <div className="modal-header">
            <div className="modal-closeIcon">
              <div onClick={()=>toggleCreateTwitterAccountModal()} className="modal-closeIcon-wrap">
                <ICON_CLOSE />
              </div>
            </div>
            <p className="modal-title">Add Twitter Account</p>
          </div>

          <div className="modal-body">
            <AddTwitterAccountForm />
          </div>
        </div>
      </div>


    </div>
  )
}

export default withRouter(Twitter)
