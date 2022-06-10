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
  setMakeSqueak,
  selectMakeSqueakStatus,
} from '../squeaks/squeaksSlice'
import {
  selectSigningProfiles,
  fetchSigningProfiles,
} from '../../features/profiles/profilesSlice'

const MakeSqueak = (props) => {
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
    console.log(values.content);

    if (!values.signingProfileId) {
      alert('Signing Profile must be selected.');
      return;
    }

    if (!values.content) {
      alert('Content cannot be empty.');
      return;
    }

    const makeValues = {
      signingProfile: values.signingProfileId,
      description: values.content,
      replyTo: props.replyToSqueak ? props.replyToSqueak.getSqueakHash() : null,
      hasRecipient: null,
      recipientProfileId: -1,
    }
    console.log('makeSqueak');
    dispatch(setMakeSqueak(makeValues))
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

  const author = props.replyToSqueak && props.replyToSqueak.getAuthor();

  const validateContent = value => {
    if (value && value.length > 280)
    return 'Content must be less than 280 characters';
  };

  const SubmitButton = () => {
    const formApi = useFormApi();

    return <button
      type="submit"
      className={'squeak-btn-side squeak-btn-active'}
      onClick={formApi.submitForm}>
      Squeak
    </button>
  };

  const MakeSqueakForm = () => (
    <Form onSubmit={submitSqueak} className="Squeak-input-side">
      <Select class="informed-select" name="signingProfileId" initialValue="">
        <Option value="" disabled>
          Select Signing Profile...
        </Option>
        {signingProfiles.map(p => {
          return <option value={p.getProfileId()}>{p.getProfileName()}</option>
        })}
      </Select>
      <TextArea class="informed-input" class="informed-textarea" name="content" validate={validateContent} placeholder="What's happening..." />
      <div className="inner-input-links">
        <div className="input-links-side">
        </div>
        <div className="squeak-btn-holder">
          <FormStateAccessor>
            {formState => (
              <div style={{ fontSize: '13px', color: formState.values.content && formState.values.content.length > 280 ? 'red' : null }}>
                {formState.values.content && formState.values.content.length > 0 && formState.values.content.length + '/280'}
              </div>
            )}
          </FormStateAccessor>
          <SubmitButton />
        </div>
      </div>
    </Form>
  );


  return (
    <>

    {/* Squeak being replied to. */}
    {props.replyToSqueak ?
      <div className="reply-content-wrapper">
        <div className="card-userPic-wrapper">
          <Link onClick={(e)=>e.stopPropagation()} to={`/app/profile/${props.replyToSqueak.getAuthorPubkey()}`}>
            <img alt="" style={{borderRadius:'50%', minWidth:'49px'}} width="100%" height="49px" src={author ? `${getProfileImageSrcString(props.replyToSqueak.getAuthor())}`: null}/>
          </Link>
        </div>
        <div className="card-content-wrapper">
          <div className="card-content-header">
            <div className="card-header-detail">
              <span className="card-header-user">
                <Link onClick={(e)=>e.stopPropagation()} to={`/app/profile/${props.replyToSqueak.getAuthorPubkey()}`}>{author ? author.getProfileName(): 'Unknown Author'}</Link>
              </span>
              <span className="card-header-username">
                <Link onClick={(e)=>e.stopPropagation()} to={`/app/profile/${props.replyToSqueak.getAuthorPubkey()}`}>{'@'+props.replyToSqueak.getAuthorPubkey()}</Link>
              </span>
              <span className="card-header-dot">·</span>
              <span className="card-header-date">
                {moment(props.replyToSqueak.getBlockTime() * 1000).fromNow()}
              </span>
            </div>
          </div>
          <div className="card-content-info">
            {props.replyToSqueak.getContentStr()}
          </div>
          <div className="reply-to-user">
            <span className="reply-squeak-username">
              Replying to
            </span>
            <span className="main-squeak-user">
              @{props.replyToSqueak.getAuthorPubkey()}
            </span>
          </div>
        </div>
      </div> : null }


      {/* New squeak content input. */}
      <div className="Squeak-input-wrapper">
        <MakeSqueakForm />
      </div>

      </>
  )
}

export default withRouter(MakeSqueak)
