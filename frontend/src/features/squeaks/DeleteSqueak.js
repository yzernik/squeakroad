import React, { useEffect, useState, useContext, useRef } from 'react'
import { withRouter } from 'react-router-dom'
import { unwrapResult } from '@reduxjs/toolkit'

import moment from 'moment'
import ContentEditable from 'react-contenteditable'
import { Link } from 'react-router-dom'
import { getProfileImageSrcString } from '../../squeakimages/images';
import Loader from '../../components/Loader'

import { Form, Input, Select, Checkbox, Relevant, Debug, TextArea, Option, useFormApi } from 'informed';

import { useSelector } from 'react-redux'
import { useDispatch } from 'react-redux'


import {
  setDeleteSqueak,
} from '../squeaks/squeaksSlice'

const DeleteSqueak = (props) => {
  const dispatch = useDispatch();

  const deleteSqueak = () => {
    console.log('deleteSqueak');
    dispatch(setDeleteSqueak(props.squeakHash));
    if (props.submittedCallback) {
      props.submittedCallback();
    }
  }

  // TODO: Show profile image for selected signing profile.
  // <div className="Squeak-profile-wrapper">
  //   {signingProfile && <img alt="" style={{ borderRadius: '50%', minWidth: '49px' }} width="100%" height="49px" src={`${getProfileImageSrcString(signingProfile)}`} />}
  // </div>

  const author = props.replyToSqueak && props.replyToSqueak.getAuthor();

  const SubmitButton = () => {
    const formApi = useFormApi();

    return <button
      type="submit"
      className={'squeak-btn-side squeak-btn-active'}
      onClick={formApi.submitForm}>
      Delete
    </button>
  };

  const DeleteSqueakForm = () => (
    <Form onSubmit={deleteSqueak} className="Squeak-input-side">
      <div className="inner-input-links">
        <div className="input-links-side">
        </div>
        <div className="squeak-btn-holder">
          <div style={{ fontSize: '13px', color: null }}>
          </div>
          <SubmitButton />
        </div>
      </div>
    </Form>
  );


  return (
    <>

      {/* New squeak content input. */}
      <div className="Squeak-input-wrapper">
        <DeleteSqueakForm />
      </div>

      </>
  )
}

export default withRouter(DeleteSqueak)
