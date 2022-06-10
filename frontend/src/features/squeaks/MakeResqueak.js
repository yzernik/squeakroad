import React, { useEffect, useState, useContext, useRef } from 'react'
import { withRouter } from 'react-router-dom'
import { unwrapResult } from '@reduxjs/toolkit'

import moment from 'moment'
import ContentEditable from 'react-contenteditable'
import { Link } from 'react-router-dom'
import { getProfileImageSrcString } from '../../squeakimages/images';
import Loader from '../../components/Loader'

import { Form, Input, Select, Checkbox, Relevant, Debug, TextArea, Option, FormStateAccessor, useFormApi } from 'informed';

import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'


import {
  setMakeResqueak,
  selectMakeResqueakStatus,
  selectMakeSqueakStatus,
} from '../squeaks/squeaksSlice'
import {
  selectSigningProfiles,
  fetchSigningProfiles,
} from '../../features/profiles/profilesSlice'

const MakeResqueak = (props) => {
  const signingProfiles = useSelector(selectSigningProfiles);
  const makeSqueakStatus = useSelector(selectMakeSqueakStatus);
  const dispatch = useDispatch();

  useEffect(() => {
    dispatch(fetchSigningProfiles());
  }, [])


  const submitSqueak = ({ values }) => {
    // TODO: toggle modal off here.
    console.log(values);
    console.log(values.signingProfileId);

    if (!values.signingProfileId) {
      alert('Signing Profile must be selected.');
      return;
    }

    if (!props.resqueakedSqueak) {
      alert('Resqueaked squeak cannot be empty.');
      return;
    }

    const makeValues = {
      signingProfile: values.signingProfileId,
      resqueakedHash: props.resqueakedSqueak.getSqueakHash(),
      replyTo: props.replyToSqueak ? props.replyToSqueak.getSqueakHash() : null,
    }
    console.log('makeResqueak');
    dispatch(setMakeResqueak(makeValues))
    .then(unwrapResult)
    .then((squeakHash) => {
      props.history.push(`/app/squeak/${squeakHash}`);
    })
    .catch((err) => {
      alert(err.message);
    });
    if (props.submittedCallback) {
      props.submittedCallback();
    }
  }

  // TODO: Show profile image for selected signing profile.
  // <div className="Squeak-profile-wrapper">
  //   {signingProfile && <img alt="" style={{ borderRadius: '50%', minWidth: '49px' }} width="100%" height="49px" src={`${getProfileImageSrcString(signingProfile)}`} />}
  // </div>

  const author = props.resqueakedSqueak && props.resqueakedSqueak.getAuthor();

  const validateContent = value => {
  if (!value || value.length > 280)
    return 'Content must be less than 280 characters';
};

const SubmitButton = () => {
  const formApi = useFormApi();

  return <button
    type="submit"
    className={'squeak-btn-side squeak-btn-active'}
    onClick={formApi.submitForm}>
    Resqueak
  </button>
};

  const MakeResqueakForm = () => (
    <Form onSubmit={submitSqueak} className="Squeak-input-side">
      <Select class="informed-select" name="signingProfileId" initialValue="">
        <Option value="" disabled>
          Select Signing Profile...
        </Option>
        {signingProfiles.map(p => {
          return <option value={p.getProfileId()}>{p.getProfileName()}</option>
        })}
      </Select>
      <div className="inner-input-links">
        <div className="input-links-side">
        </div>
        <div className="squeak-btn-holder">
          <SubmitButton />
        </div>
      </div>
    </Form>
  );


  return (
    <>

    {/* Squeak being resqueaked. */}
    {props.resqueakedSqueak ?
      <div className="reply-content-wrapper">
        <div className="card-userPic-wrapper">
          <Link onClick={(e)=>e.stopPropagation()} to={`/app/profile/${props.resqueakedSqueak.getAuthorPubkey()}`}>
            <img alt="" style={{borderRadius:'50%', minWidth:'49px'}} width="100%" height="49px" src={author ? `${getProfileImageSrcString(props.resqueakedSqueak.getAuthor())}`: null}/>
          </Link>
        </div>
        <div className="card-content-wrapper">
          <div className="card-content-header">
            <div className="card-header-detail">
              <span className="card-header-user">
                <Link onClick={(e)=>e.stopPropagation()} to={`/app/profile/${props.resqueakedSqueak.getAuthorPubkey()}`}>{author ? author.getProfileName(): 'Unknown Author'}</Link>
              </span>
              <span className="card-header-username">
                <Link onClick={(e)=>e.stopPropagation()} to={`/app/profile/${props.resqueakedSqueak.getAuthorPubkey()}`}>{'@'+props.resqueakedSqueak.getAuthorPubkey()}</Link>
              </span>
              <span className="card-header-dot">·</span>
              <span className="card-header-date">
                {moment(props.resqueakedSqueak.getBlockTime() * 1000).fromNow()}
              </span>
            </div>
          </div>
          <div className="card-content-info">
            {props.resqueakedSqueak.getContentStr()}
          </div>
          <div className="reply-to-user">
            <span className="reply-squeak-username">
              Replying to
            </span>
            <span className="main-squeak-user">
              @{props.resqueakedSqueak.getAuthorPubkey()}
            </span>
          </div>
        </div>
      </div> : null }


      {/* New squeak content input. */}
      <div className="Squeak-input-wrapper">
        <MakeResqueakForm />
      </div>

      </>
  )
}

export default withRouter(MakeResqueak)
